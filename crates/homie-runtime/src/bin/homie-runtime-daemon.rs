use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::Arc;

use homie_runtime::daemon::{
    DaemonError, DaemonLease, canonical_daemon_executables, daemon_instance_id, executable_sha256,
};
use homie_runtime::dispatcher::{RuntimeDispatcher, RuntimeLongRunningExecutor};
use homie_runtime::long_running::LongRunningLane;
use homie_runtime::runtime_actor::{
    RuntimeActor, RuntimeActorHandle, RuntimeSupervisorBackend, ServiceError,
};
use homie_runtime::terminal_stream::RuntimeTerminalBackend;
use homie_runtime::{
    RuntimeConfig, RuntimeEventWaitHandler, RuntimeServer, RuntimeStreamHandler, RuntimeSupervisor,
    ServerConfig, ServerIdentity,
};

const PREPARE_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(10);

#[tokio::main]
async fn main() -> ExitCode {
    let result = match parse_data_dir() {
        Ok(data_dir) => run_daemon(data_dir).await,
        Err(()) => Err(()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => {
            eprintln!("homie-runtime-daemon: startup failed");
            ExitCode::FAILURE
        }
    }
}

fn parse_data_dir() -> Result<PathBuf, ()> {
    let mut arguments = std::env::args_os().skip(1);
    if arguments.next().as_deref() != Some(std::ffi::OsStr::new("--data-dir")) {
        return Err(());
    }
    let data_dir = PathBuf::from(arguments.next().ok_or(())?);
    if !data_dir.is_absolute() || arguments.next().is_some() {
        return Err(());
    }
    Ok(data_dir)
}

async fn run_daemon(data_dir: PathBuf) -> Result<(), ()> {
    let mut lease = match DaemonLease::acquire(&data_dir) {
        Ok(lease) => lease,
        Err(DaemonError::AlreadyRunning) => return Ok(()),
        Err(_) => return Err(()),
    };
    let current_exe = std::env::current_exe().map_err(|_| ())?;
    let (daemon_executable, holder_executable) =
        canonical_daemon_executables(current_exe).map_err(|_| ())?;
    let executable_hash = executable_sha256(&daemon_executable)
        .await
        .map_err(|_| ())?;
    let supervisor = RuntimeSupervisor::open_with_holder(
        RuntimeConfig {
            data_dir: lease.paths().data_dir.clone(),
        },
        holder_executable,
    )
    .map_err(|_| ())?;
    let event_store = supervisor.event_store();
    let mut interrupt = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .map_err(|_| ())?;
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|_| ())?;
    let actor = RuntimeActor::spawn(RuntimeSupervisorBackend::new(supervisor)).map_err(|_| ())?;
    let lane = match LongRunningLane::spawn() {
        Ok(lane) => lane,
        Err(_) => {
            let _ = actor.shutdown_async().await;
            return Err(());
        }
    };
    let listener = match lease.bind() {
        Ok(listener) => listener,
        Err(_) => {
            let _ = lane.shutdown_async().await;
            let _ = actor.shutdown_async().await;
            return Err(());
        }
    };

    let actor_handle = actor.handle();
    let shutdown_actor = actor_handle.clone();
    let dispatcher = Arc::new(RuntimeDispatcher::new(
        actor_handle.clone(),
        lane.handle(),
        Arc::new(RuntimeLongRunningExecutor),
        Arc::new(RuntimeEventWaitHandler::new(event_store.clone())),
    ));
    let streams = Arc::new(RuntimeStreamHandler::new(
        event_store,
        Arc::new(RuntimeTerminalBackend::new(actor_handle)),
    ));
    let server = Arc::new(RuntimeServer::new(
        ServerConfig::current_user(),
        ServerIdentity {
            daemon_build: concat!("homie-runtime/", env!("CARGO_PKG_VERSION")).to_string(),
            daemon_version: env!("CARGO_PKG_VERSION").to_string(),
            daemon_pid: std::process::id(),
            daemon_instance_id: daemon_instance_id(),
            executable_hash,
        },
        dispatcher,
        streams,
    ));

    let shutdown = server.shutdown_handle();
    let serve = server.serve_listener(listener);
    tokio::pin!(serve);
    let mut prepare_result = Ok(());
    let serve_result = tokio::select! {
        result = &mut serve => result,
        () = shutdown_signal(&mut interrupt, &mut terminate) => {
            prepare_result = prepare_actor(&shutdown_actor).await;
            shutdown.request_shutdown();
            serve.await
        }
    };
    let lane_result = lane.shutdown_async().await;
    let actor_result = actor.shutdown_async().await;
    drop(lease);
    if prepare_result.is_err()
        || serve_result.is_err()
        || lane_result.is_err()
        || actor_result.is_err()
    {
        return Err(());
    }
    Ok(())
}

async fn shutdown_signal(
    interrupt: &mut tokio::signal::unix::Signal,
    terminate: &mut tokio::signal::unix::Signal,
) {
    tokio::select! {
        _ = interrupt.recv() => {}
        _ = terminate.recv() => {}
    }
}

async fn prepare_actor(actor: &RuntimeActorHandle) -> Result<(), ServiceError> {
    loop {
        let result = match actor.prepare_shutdown() {
            Ok(receiver) => receiver.await.unwrap_or(Err(ServiceError::Unavailable)),
            Err(error) => Err(error),
        };
        match result {
            Ok(()) | Err(ServiceError::Unavailable) => return Ok(()),
            Err(ServiceError::Backpressure) => tokio::time::sleep(PREPARE_RETRY_DELAY).await,
            Err(error) => return Err(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;
    use std::time::Duration;

    use homie_runtime::dispatcher::{ActorRequest, RuntimeResponse};
    use homie_runtime::runtime_actor::{
        ACTOR_QUEUE_CAPACITY, RuntimeBackend, RuntimeCall, RuntimeReply, ServiceResult,
    };

    use super::*;

    struct GatedBackend {
        gate: Arc<(Mutex<bool>, Condvar)>,
    }

    impl RuntimeBackend for GatedBackend {
        fn call(&mut self, _request: RuntimeCall) -> ServiceResult<RuntimeReply> {
            let (open, changed) = &*self.gate;
            let mut open = open.lock().expect("gate");
            while !*open {
                open = changed.wait(open).expect("gate wait");
            }
            Ok(ack())
        }
    }

    struct PrepareErrorBackend(ServiceError);

    impl RuntimeBackend for PrepareErrorBackend {
        fn call(&mut self, _request: RuntimeCall) -> ServiceResult<RuntimeReply> {
            Ok(ack())
        }

        fn prepare_shutdown(&mut self) -> ServiceResult<()> {
            Err(self.0.clone())
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn prepare_actor_retries_without_busy_spin_until_full_queue_accepts() {
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let actor = RuntimeActor::spawn(GatedBackend { gate: gate.clone() }).expect("actor");
        let actor_handle = actor.handle();
        let running = actor_handle
            .try_call(RuntimeCall::Invoke(ActorRequest::SessionList))
            .expect("running call");
        thread::sleep(Duration::from_millis(20));
        let pending = (0..ACTOR_QUEUE_CAPACITY)
            .map(|_| {
                actor_handle
                    .try_call(RuntimeCall::Invoke(ActorRequest::SessionList))
                    .expect("pending call")
            })
            .collect::<Vec<_>>();
        let prepare_handle = actor_handle.clone();
        let prepare = tokio::spawn(async move { prepare_actor(&prepare_handle).await });

        tokio::time::sleep(Duration::from_millis(25)).await;
        let was_pending = !prepare.is_finished();
        let (open, changed) = &*gate;
        *open.lock().expect("gate") = true;
        changed.notify_all();

        let prepare_result = tokio::time::timeout(Duration::from_secs(2), prepare)
            .await
            .expect("bounded prepare completion")
            .expect("prepare task");
        drop(running);
        drop(pending);
        actor.shutdown_async().await.expect("actor shutdown");
        assert!(was_pending, "prepare must asynchronously retry");
        prepare_result.expect("prepare success");
    }

    #[tokio::test]
    async fn prepare_actor_treats_unavailable_as_already_draining() {
        let actor =
            RuntimeActor::spawn(PrepareErrorBackend(ServiceError::Internal)).expect("actor");
        let actor_handle = actor.handle();
        actor.shutdown_async().await.expect("actor shutdown");

        prepare_actor(&actor_handle)
            .await
            .expect("stopped actor is already draining");
    }

    #[tokio::test]
    async fn prepare_actor_propagates_non_retryable_errors() {
        let actor =
            RuntimeActor::spawn(PrepareErrorBackend(ServiceError::Internal)).expect("actor");

        let error = prepare_actor(&actor.handle())
            .await
            .expect_err("internal prepare failure");

        assert_eq!(error, ServiceError::Internal);
        actor.shutdown_async().await.expect("actor shutdown");
    }

    fn ack() -> RuntimeReply {
        RuntimeReply::Response(RuntimeResponse::Ack(homie_proto::transport::AckResult {
            ok: true,
        }))
    }
}
