use std::ffi::{OsStr, OsString};
use std::io;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use tokio::sync::oneshot;

use crate::runtime_actor::{ServiceError, ServiceResult};

pub const LONG_RUNNING_QUEUE_CAPACITY: usize = 32;

#[derive(Clone, Default)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum JobKind {
    ReadOnly,
    Mutation,
}

pub struct JobOptions {
    deadline: Instant,
    cancellation: CancellationToken,
    kind: JobKind,
}

impl JobOptions {
    #[must_use]
    pub fn read_only(timeout: Duration) -> Self {
        Self::new(timeout, JobKind::ReadOnly)
    }

    #[must_use]
    pub fn mutation(timeout: Duration) -> Self {
        Self::new(timeout, JobKind::Mutation)
    }

    #[must_use]
    pub fn with_cancellation(mut self, cancellation: CancellationToken) -> Self {
        self.cancellation = cancellation;
        self
    }

    fn new(timeout: Duration, kind: JobKind) -> Self {
        Self {
            deadline: Instant::now() + timeout,
            cancellation: CancellationToken::new(),
            kind,
        }
    }
}

pub struct JobContext {
    deadline: Instant,
    cancellation: CancellationToken,
    lane_shutdown: CancellationToken,
    ignore_caller_cancellation: bool,
}

impl JobContext {
    pub fn checkpoint(&self) -> ServiceResult<()> {
        if Instant::now() >= self.deadline {
            return Err(ServiceError::Timeout);
        }
        if !self.ignore_caller_cancellation
            && (self.cancellation.is_cancelled() || self.lane_shutdown.is_cancelled())
        {
            return Err(ServiceError::Cancelled);
        }
        Ok(())
    }

    #[must_use]
    pub fn remaining(&self) -> Duration {
        self.deadline.saturating_duration_since(Instant::now())
    }
}

trait LaneJob: Send {
    fn execute(self: Box<Self>);
}

struct Job<T, F> {
    options: JobOptions,
    lane_shutdown: CancellationToken,
    execute: F,
    reply: oneshot::Sender<ServiceResult<T>>,
}

impl<T, F> LaneJob for Job<T, F>
where
    T: Send + 'static,
    F: FnOnce(&JobContext) -> ServiceResult<T> + Send + 'static,
{
    fn execute(self: Box<Self>) {
        execute_job(*self);
    }
}

enum LaneCommand {
    Run(Box<dyn LaneJob>),
    Shutdown { reply: oneshot::Sender<()> },
}

#[derive(Clone)]
pub struct LongRunningLaneHandle {
    sender: mpsc::SyncSender<LaneCommand>,
    accepting: Arc<AtomicBool>,
    shutdown: CancellationToken,
}

impl LongRunningLaneHandle {
    pub fn try_submit<T>(
        &self,
        options: JobOptions,
        execute: impl FnOnce(&JobContext) -> ServiceResult<T> + Send + 'static,
    ) -> ServiceResult<oneshot::Receiver<ServiceResult<T>>>
    where
        T: Send + 'static,
    {
        if !self.accepting.load(Ordering::Acquire) {
            return Err(ServiceError::Unavailable);
        }
        let (reply, receiver) = oneshot::channel();
        let job = Job {
            options,
            lane_shutdown: self.shutdown.clone(),
            execute: Box::new(execute),
            reply,
        };
        self.sender
            .try_send(LaneCommand::Run(Box::new(job)))
            .map_err(map_try_send_error)?;
        Ok(receiver)
    }
}

pub struct LongRunningLane {
    handle: LongRunningLaneHandle,
    join: JoinHandle<()>,
}

impl LongRunningLane {
    pub fn spawn() -> io::Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(LONG_RUNNING_QUEUE_CAPACITY);
        let accepting = Arc::new(AtomicBool::new(true));
        let worker_accepting = accepting.clone();
        let shutdown = CancellationToken::new();
        let join = thread::Builder::new()
            .name("homie-runtime-long-running".to_string())
            .spawn(move || run_lane(receiver, worker_accepting))?;
        Ok(Self {
            handle: LongRunningLaneHandle {
                sender,
                accepting,
                shutdown,
            },
            join,
        })
    }

    #[must_use]
    pub fn handle(&self) -> LongRunningLaneHandle {
        self.handle.clone()
    }

    pub fn shutdown(self) -> ServiceResult<()> {
        self.handle.accepting.store(false, Ordering::Release);
        self.handle.shutdown.cancel();
        let (reply, receiver) = oneshot::channel();
        self.handle
            .sender
            .send(LaneCommand::Shutdown { reply })
            .map_err(|_| ServiceError::Unavailable)?;
        receiver
            .blocking_recv()
            .map_err(|_| ServiceError::Unavailable)?;
        self.join.join().map_err(|_| ServiceError::Internal)
    }

    pub async fn shutdown_async(self) -> ServiceResult<()> {
        tokio::task::spawn_blocking(move || self.shutdown())
            .await
            .map_err(|_| ServiceError::Internal)?
    }
}

fn run_lane(receiver: mpsc::Receiver<LaneCommand>, accepting: Arc<AtomicBool>) {
    while let Ok(command) = receiver.recv() {
        match command {
            LaneCommand::Run(job) => job.execute(),
            LaneCommand::Shutdown { reply } => {
                accepting.store(false, Ordering::Release);
                let _ = reply.send(());
                break;
            }
        }
    }
    accepting.store(false, Ordering::Release);
}

fn execute_job<T, F>(job: Job<T, F>)
where
    F: FnOnce(&JobContext) -> ServiceResult<T>,
{
    let result = if Instant::now() >= job.options.deadline {
        Err(ServiceError::Timeout)
    } else if job.options.cancellation.is_cancelled()
        || job.lane_shutdown.is_cancelled()
        || job.reply.is_closed()
    {
        Err(ServiceError::Cancelled)
    } else {
        let context = JobContext {
            deadline: job.options.deadline,
            cancellation: job.options.cancellation,
            lane_shutdown: job.lane_shutdown,
            ignore_caller_cancellation: job.options.kind == JobKind::Mutation,
        };
        (job.execute)(&context)
    };
    let _ = job.reply.send(result);
}

fn map_try_send_error<T>(error: mpsc::TrySendError<T>) -> ServiceError {
    match error {
        mpsc::TrySendError::Full(_) => ServiceError::Backpressure,
        mpsc::TrySendError::Disconnected(_) => ServiceError::Unavailable,
    }
}

pub(crate) struct BoundedCommand {
    program: PathBuf,
    args: Vec<OsString>,
    cwd: Option<PathBuf>,
    clear_env: bool,
    env: Vec<(OsString, OsString)>,
    stdout_limit: usize,
    stderr_limit: usize,
}

impl BoundedCommand {
    pub(crate) fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: None,
            clear_env: false,
            env: Vec::new(),
            stdout_limit: 1024 * 1024,
            stderr_limit: 256 * 1024,
        }
    }

    pub(crate) fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.args
            .extend(args.into_iter().map(|arg| arg.as_ref().to_os_string()));
        self
    }

    pub(crate) fn current_dir(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    pub(crate) fn env_clear(mut self) -> Self {
        self.clear_env = true;
        self
    }

    pub(crate) fn env(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        self.env.push((key.into(), value.into()));
        self
    }

    pub(crate) fn stdout_limit(mut self, limit: usize) -> Self {
        self.stdout_limit = limit;
        self
    }

    pub(crate) fn stderr_limit(mut self, limit: usize) -> Self {
        self.stderr_limit = limit;
        self
    }
}

pub(crate) struct BoundedOutput {
    pub(crate) status: ExitStatus,
    pub(crate) stdout: Vec<u8>,
    pub(crate) stderr: Vec<u8>,
    pub(crate) stdout_truncated: bool,
    pub(crate) stderr_truncated: bool,
}

type CapturedPipe = (Vec<u8>, bool);
type PipeReadResult = io::Result<CapturedPipe>;
type PipeReaderJoin = JoinHandle<PipeReadResult>;

pub(crate) fn run_bounded_command(
    context: &JobContext,
    spec: BoundedCommand,
) -> ServiceResult<BoundedOutput> {
    run_bounded_command_with(context, spec, |child| child.try_wait(), spawn_pipe_reader)
}

fn run_bounded_command_with<TryWait, SpawnReader>(
    context: &JobContext,
    spec: BoundedCommand,
    mut try_wait: TryWait,
    mut spawn_reader: SpawnReader,
) -> ServiceResult<BoundedOutput>
where
    TryWait: FnMut(&mut std::process::Child) -> io::Result<Option<ExitStatus>>,
    SpawnReader: FnMut(Box<dyn Read + Send>, usize) -> io::Result<PipeReaderJoin>,
{
    context.checkpoint()?;
    let mut command = Command::new(&spec.program);
    command
        .args(&spec.args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some(cwd) = &spec.cwd {
        command.current_dir(cwd);
    }
    if spec.clear_env {
        command.env_clear();
    }
    command.envs(spec.env);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }

    let mut child = command.spawn().map_err(|_| ServiceError::Internal)?;
    let Some(stdout) = child.stdout.take() else {
        terminate_process_group(&mut child);
        return Err(ServiceError::Internal);
    };
    let Some(stderr) = child.stderr.take() else {
        terminate_process_group(&mut child);
        return Err(ServiceError::Internal);
    };
    let stdout_reader = match spawn_reader(Box::new(stdout), spec.stdout_limit) {
        Ok(reader) => reader,
        Err(_) => {
            terminate_process_group(&mut child);
            return Err(ServiceError::Internal);
        }
    };
    let mut stdout_reader = PipeReaderState::new(stdout_reader);
    let stderr_reader = match spawn_reader(Box::new(stderr), spec.stderr_limit) {
        Ok(reader) => reader,
        Err(_) => {
            terminate_process_group(&mut child);
            stdout_reader.join_ignoring_result();
            return Err(ServiceError::Internal);
        }
    };
    let mut stderr_reader = PipeReaderState::new(stderr_reader);

    let status = loop {
        let status = match try_wait(&mut child) {
            Ok(status) => status,
            Err(_) => {
                cleanup_spawned_command(&mut child, &mut stdout_reader, &mut stderr_reader);
                return Err(ServiceError::Internal);
            }
        };
        match status {
            Some(status) => break status,
            None => {
                if let Err(error) = context.checkpoint() {
                    cleanup_spawned_command(&mut child, &mut stdout_reader, &mut stderr_reader);
                    return Err(error);
                }
                if stdout_reader.poll().is_err() || stderr_reader.poll().is_err() {
                    cleanup_spawned_command(&mut child, &mut stdout_reader, &mut stderr_reader);
                    return Err(ServiceError::Internal);
                }
                thread::sleep(Duration::from_millis(10));
            }
        }
    };
    while !stdout_reader.is_complete() || !stderr_reader.is_complete() {
        if let Err(error) = context.checkpoint() {
            cleanup_spawned_command(&mut child, &mut stdout_reader, &mut stderr_reader);
            return Err(error);
        }
        if stdout_reader.poll().is_err() || stderr_reader.poll().is_err() {
            cleanup_spawned_command(&mut child, &mut stdout_reader, &mut stderr_reader);
            return Err(ServiceError::Internal);
        }
        if !stdout_reader.is_complete() || !stderr_reader.is_complete() {
            thread::sleep(Duration::from_millis(10));
        }
    }
    let (stdout, stdout_truncated) = match stdout_reader.finish() {
        Ok(output) => output,
        Err(error) => {
            cleanup_spawned_command(&mut child, &mut stdout_reader, &mut stderr_reader);
            return Err(error);
        }
    };
    let (stderr, stderr_truncated) = match stderr_reader.finish() {
        Ok(output) => output,
        Err(error) => {
            cleanup_spawned_command(&mut child, &mut stdout_reader, &mut stderr_reader);
            return Err(error);
        }
    };
    Ok(BoundedOutput {
        status,
        stdout,
        stderr,
        stdout_truncated,
        stderr_truncated,
    })
}

fn spawn_pipe_reader(reader: Box<dyn Read + Send>, limit: usize) -> io::Result<PipeReaderJoin> {
    thread::Builder::new()
        .spawn(move || read_capped(reader, limit))
        .map_err(|error| io::Error::other(error.to_string()))
}

fn read_capped(mut reader: impl Read, limit: usize) -> PipeReadResult {
    let mut captured = Vec::with_capacity(limit.min(64 * 1024));
    let mut truncated = false;
    let mut buffer = [0_u8; 8192];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        let remaining = limit.saturating_sub(captured.len());
        let keep = read.min(remaining);
        captured.extend_from_slice(&buffer[..keep]);
        truncated |= keep < read;
    }
    Ok((captured, truncated))
}

struct PipeReaderState {
    join: Option<PipeReaderJoin>,
    output: Option<CapturedPipe>,
}

impl PipeReaderState {
    fn new(join: PipeReaderJoin) -> Self {
        Self {
            join: Some(join),
            output: None,
        }
    }

    fn poll(&mut self) -> ServiceResult<()> {
        if self.join.as_ref().is_some_and(JoinHandle::is_finished) {
            self.output = Some(self.join_reader()?);
        }
        Ok(())
    }

    fn is_complete(&self) -> bool {
        self.output.is_some()
    }

    fn finish(&mut self) -> ServiceResult<CapturedPipe> {
        if let Some(output) = self.output.take() {
            return Ok(output);
        }
        self.join_reader()
    }

    fn join_reader(&mut self) -> ServiceResult<CapturedPipe> {
        self.join
            .take()
            .ok_or(ServiceError::Internal)?
            .join()
            .map_err(|_| ServiceError::Internal)?
            .map_err(|_| ServiceError::Internal)
    }

    fn join_ignoring_result(&mut self) {
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn cleanup_spawned_command(
    child: &mut std::process::Child,
    stdout_reader: &mut PipeReaderState,
    stderr_reader: &mut PipeReaderState,
) {
    terminate_process_group(child);
    stdout_reader.join_ignoring_result();
    stderr_reader.join_ignoring_result();
}

fn terminate_process_group(child: &mut std::process::Child) {
    #[cfg(unix)]
    {
        let pid = child.id() as i32;
        // SAFETY: the child was spawned into a process group whose id is its pid.
        let result = unsafe { libc::kill(-pid, libc::SIGKILL) };
        if result < 0 && io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
            let _ = child.kill();
        }
    }
    #[cfg(not(unix))]
    {
        let _ = child.kill();
    }
    let _ = child.wait();
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{Arc, Condvar, Mutex};
    use std::thread;
    use std::time::Duration;

    use serde_json::json;

    use super::*;

    #[test]
    fn jobs_are_serialized_on_the_named_worker() {
        let lane = LongRunningLane::spawn().expect("spawn lane");
        let handle = lane.handle();
        let active = Arc::new(AtomicUsize::new(0));
        let max_active = Arc::new(AtomicUsize::new(0));

        let replies = (0..8)
            .map(|_| {
                let active = active.clone();
                let max_active = max_active.clone();
                handle
                    .try_submit(JobOptions::read_only(Duration::from_secs(1)), move |_| {
                        let now = active.fetch_add(1, Ordering::SeqCst) + 1;
                        max_active.fetch_max(now, Ordering::SeqCst);
                        let name = thread::current().name().unwrap_or_default().to_string();
                        thread::sleep(Duration::from_millis(5));
                        active.fetch_sub(1, Ordering::SeqCst);
                        Ok(json!(name))
                    })
                    .expect("submit")
            })
            .collect::<Vec<_>>();

        for reply in replies {
            assert_eq!(
                reply.blocking_recv().expect("reply").expect("job"),
                json!("homie-runtime-long-running")
            );
        }
        assert_eq!(max_active.load(Ordering::SeqCst), 1);
        lane.shutdown().expect("shutdown");
    }

    #[test]
    fn the_33rd_pending_job_is_rejected_with_backpressure() {
        let lane = LongRunningLane::spawn().expect("spawn lane");
        let handle = lane.handle();
        let gate = Arc::new((Mutex::new((false, false)), Condvar::new()));
        let worker_gate = gate.clone();
        let running = handle
            .try_submit(JobOptions::read_only(Duration::from_secs(2)), move |_| {
                let (state, changed) = &*worker_gate;
                let mut state = state.lock().expect("gate");
                state.0 = true;
                changed.notify_all();
                while !state.1 {
                    state = changed.wait(state).expect("wait");
                }
                Ok(json!(true))
            })
            .expect("running");
        let (state, changed) = &*gate;
        let mut state = state.lock().expect("gate");
        while !state.0 {
            state = changed.wait(state).expect("started");
        }
        drop(state);

        let pending = (0..LONG_RUNNING_QUEUE_CAPACITY)
            .map(|_| {
                handle
                    .try_submit(JobOptions::read_only(Duration::from_secs(2)), |_| {
                        Ok(json!(true))
                    })
                    .expect("pending")
            })
            .collect::<Vec<_>>();
        assert_eq!(
            handle
                .try_submit(JobOptions::read_only(Duration::from_secs(2)), |_| Ok(
                    json!(true)
                ))
                .expect_err("full"),
            ServiceError::Backpressure
        );

        let mut state = gate.0.lock().expect("gate");
        state.1 = true;
        gate.1.notify_all();
        drop(state);
        running
            .blocking_recv()
            .expect("running reply")
            .expect("job");
        for reply in pending {
            reply.blocking_recv().expect("pending reply").expect("job");
        }
        lane.shutdown().expect("shutdown");
    }

    #[test]
    fn an_expired_queued_job_never_executes() {
        let lane = LongRunningLane::spawn().expect("spawn lane");
        let handle = lane.handle();
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let worker_gate = gate.clone();
        let running = handle
            .try_submit(JobOptions::read_only(Duration::from_secs(1)), move |_| {
                let (open, changed) = &*worker_gate;
                let mut open = open.lock().expect("gate");
                while !*open {
                    open = changed.wait(open).expect("wait");
                }
                Ok(json!(true))
            })
            .expect("running");
        thread::sleep(Duration::from_millis(20));

        let executed = Arc::new(AtomicBool::new(false));
        let job_executed = executed.clone();
        let expired = handle
            .try_submit(
                JobOptions::read_only(Duration::from_millis(10)),
                move |_| {
                    job_executed.store(true, Ordering::SeqCst);
                    Ok(json!(true))
                },
            )
            .expect("expired");
        thread::sleep(Duration::from_millis(20));
        *gate.0.lock().expect("gate") = true;
        gate.1.notify_all();

        running
            .blocking_recv()
            .expect("running reply")
            .expect("job");
        assert_eq!(
            expired.blocking_recv().expect("expired reply"),
            Err(ServiceError::Timeout)
        );
        assert!(!executed.load(Ordering::SeqCst));
        lane.shutdown().expect("shutdown");
    }

    #[test]
    fn a_cancelled_queued_read_job_never_executes() {
        let lane = LongRunningLane::spawn().expect("spawn lane");
        let handle = lane.handle();
        let gate = Arc::new((Mutex::new(false), Condvar::new()));
        let worker_gate = gate.clone();
        let running = handle
            .try_submit(JobOptions::read_only(Duration::from_secs(1)), move |_| {
                let (open, changed) = &*worker_gate;
                let mut open = open.lock().expect("gate");
                while !*open {
                    open = changed.wait(open).expect("wait");
                }
                Ok(json!(true))
            })
            .expect("running");
        thread::sleep(Duration::from_millis(20));

        let cancellation = CancellationToken::new();
        let executed = Arc::new(AtomicBool::new(false));
        let job_executed = executed.clone();
        let cancelled = handle
            .try_submit(
                JobOptions::read_only(Duration::from_secs(1))
                    .with_cancellation(cancellation.clone()),
                move |_| {
                    job_executed.store(true, Ordering::SeqCst);
                    Ok(json!(true))
                },
            )
            .expect("cancelled");
        cancellation.cancel();
        *gate.0.lock().expect("gate") = true;
        gate.1.notify_all();

        running
            .blocking_recv()
            .expect("running reply")
            .expect("job");
        assert_eq!(
            cancelled.blocking_recv().expect("cancelled reply"),
            Err(ServiceError::Cancelled)
        );
        assert!(!executed.load(Ordering::SeqCst));
        lane.shutdown().expect("shutdown");
    }

    #[test]
    fn a_started_mutation_ignores_caller_cancellation() {
        let lane = LongRunningLane::spawn().expect("spawn lane");
        let handle = lane.handle();
        let cancellation = CancellationToken::new();
        let gate = Arc::new((Mutex::new((false, false)), Condvar::new()));
        let worker_gate = gate.clone();
        let reply = handle
            .try_submit(
                JobOptions::mutation(Duration::from_secs(1))
                    .with_cancellation(cancellation.clone()),
                move |context| {
                    let (state, changed) = &*worker_gate;
                    let mut state = state.lock().expect("gate");
                    state.0 = true;
                    changed.notify_all();
                    while !state.1 {
                        state = changed.wait(state).expect("wait");
                    }
                    context.checkpoint()?;
                    Ok(json!("committed"))
                },
            )
            .expect("submit");
        let (state, changed) = &*gate;
        let mut state = state.lock().expect("gate");
        while !state.0 {
            state = changed.wait(state).expect("started");
        }
        cancellation.cancel();
        state.1 = true;
        changed.notify_all();
        drop(state);

        assert_eq!(
            reply.blocking_recv().expect("reply").expect("mutation"),
            json!("committed")
        );
        lane.shutdown().expect("shutdown");
    }

    #[test]
    fn shutdown_cancels_a_started_cooperative_read_job() {
        let lane = LongRunningLane::spawn().expect("spawn lane");
        let handle = lane.handle();
        let started = Arc::new((Mutex::new(false), Condvar::new()));
        let worker_started = started.clone();
        let reply = handle
            .try_submit::<serde_json::Value>(
                JobOptions::read_only(Duration::from_secs(5)),
                move |context| {
                    *worker_started.0.lock().expect("started") = true;
                    worker_started.1.notify_all();
                    loop {
                        context.checkpoint()?;
                        thread::sleep(Duration::from_millis(5));
                    }
                },
            )
            .expect("submit read");
        let mut observed = started.0.lock().expect("started");
        while !*observed {
            observed = started.1.wait(observed).expect("wait started");
        }
        drop(observed);

        lane.shutdown().expect("shutdown");

        assert_eq!(
            reply.blocking_recv().expect("read reply"),
            Err(ServiceError::Cancelled)
        );
    }

    #[test]
    fn shutdown_waits_for_a_started_mutation_after_its_caller_disconnects() {
        let lane = LongRunningLane::spawn().expect("spawn lane");
        let handle = lane.handle();
        let gate = Arc::new((Mutex::new((false, false)), Condvar::new()));
        let worker_gate = gate.clone();
        let committed = Arc::new(AtomicBool::new(false));
        let mutation_committed = committed.clone();
        let mutation = handle
            .try_submit(JobOptions::mutation(Duration::from_secs(5)), move |_| {
                let mut state = worker_gate.0.lock().expect("gate");
                state.0 = true;
                worker_gate.1.notify_all();
                while !state.1 {
                    state = worker_gate.1.wait(state).expect("wait release");
                }
                mutation_committed.store(true, Ordering::SeqCst);
                Ok(json!("committed"))
            })
            .expect("running mutation");
        let mut observed = gate.0.lock().expect("gate");
        while !observed.0 {
            observed = gate.1.wait(observed).expect("wait started");
        }
        drop(observed);

        drop(mutation);
        let shutdown = thread::spawn(move || lane.shutdown());
        thread::sleep(Duration::from_millis(20));
        assert!(
            !shutdown.is_finished(),
            "shutdown returned before the started mutation completed"
        );

        let mut state = gate.0.lock().expect("gate");
        state.1 = true;
        gate.1.notify_all();
        drop(state);
        shutdown.join().expect("shutdown thread").expect("shutdown");
        assert!(committed.load(Ordering::SeqCst));
    }

    #[cfg(unix)]
    #[test]
    fn a_monitor_error_terminates_and_reaps_the_process_group_and_joins_readers() {
        let temp = tempfile::tempdir().expect("tempdir");
        let shell_pid_path = temp.path().join("shell.pid");
        let child_pid_path = temp.path().join("child.pid");
        let script = process_group_script(&shell_pid_path, &child_pid_path, true);
        let readers_finished = Arc::new(AtomicUsize::new(0));
        let reader_counter = readers_finished.clone();
        let monitor_shell_pid_path = shell_pid_path.clone();
        let monitor_child_pid_path = child_pid_path.clone();
        let context = read_only_context(Duration::from_secs(2));

        let result = run_bounded_command_with(
            &context,
            BoundedCommand::new("/bin/sh").args(["-c", script.as_str()]),
            move |_child| {
                wait_for_pid(&monitor_shell_pid_path);
                wait_for_pid(&monitor_child_pid_path);
                Err(io::Error::other("forced monitor error"))
            },
            move |reader: Box<dyn Read + Send>, limit| {
                let reader_counter = reader_counter.clone();
                thread::Builder::new()
                    .spawn(move || {
                        let result = read_capped(reader, limit);
                        thread::sleep(Duration::from_millis(50));
                        reader_counter.fetch_add(1, Ordering::SeqCst);
                        result
                    })
                    .map_err(|error| io::Error::other(error.to_string()))
            },
        );

        let shell_pid = wait_for_pid(&shell_pid_path);
        let child_pid = wait_for_pid(&child_pid_path);
        let processes_reaped = wait_for_processes_to_exit(shell_pid, child_pid);
        let observed = (
            matches!(result, Err(ServiceError::Internal)),
            readers_finished.load(Ordering::SeqCst),
            processes_reaped,
        );
        kill_process_group_for_test(shell_pid);

        assert_eq!(observed, (true, 2, true));
    }

    #[cfg(unix)]
    #[test]
    fn a_pipe_read_error_is_internal_and_cleans_up_the_spawned_command() {
        struct FailingReader;

        impl Read for FailingReader {
            fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("forced pipe read error"))
            }
        }

        let temp = tempfile::tempdir().expect("tempdir");
        let shell_pid_path = temp.path().join("shell.pid");
        let child_pid_path = temp.path().join("child.pid");
        let script = process_group_script(&shell_pid_path, &child_pid_path, true);
        let readers_finished = Arc::new(AtomicUsize::new(0));
        let mut reader_index = 0;
        let context = read_only_context(Duration::from_millis(300));
        let failing_shell_pid_path = shell_pid_path.clone();
        let failing_child_pid_path = child_pid_path.clone();

        let result = run_bounded_command_with(
            &context,
            BoundedCommand::new("/bin/sh").args(["-c", script.as_str()]),
            |child| child.try_wait(),
            {
                let readers_finished = readers_finished.clone();
                move |reader: Box<dyn Read + Send>, limit| {
                    let index = reader_index;
                    reader_index += 1;
                    let readers_finished = readers_finished.clone();
                    let shell_pid_path = failing_shell_pid_path.clone();
                    let child_pid_path = failing_child_pid_path.clone();
                    thread::Builder::new()
                        .spawn(move || {
                            let result = if index == 0 {
                                wait_for_pid(&shell_pid_path);
                                wait_for_pid(&child_pid_path);
                                drop(reader);
                                read_capped(FailingReader, limit)
                            } else {
                                read_capped(reader, limit)
                            };
                            readers_finished.fetch_add(1, Ordering::SeqCst);
                            result
                        })
                        .map_err(|error| io::Error::other(error.to_string()))
                }
            },
        );

        let shell_pid = wait_for_pid(&shell_pid_path);
        let child_pid = wait_for_pid(&child_pid_path);
        let processes_reaped = wait_for_processes_to_exit(shell_pid, child_pid);
        let observed = (
            matches!(result, Err(ServiceError::Internal)),
            readers_finished.load(Ordering::SeqCst),
            processes_reaped,
        );
        kill_process_group_for_test(shell_pid);

        assert_eq!(observed, (true, 2, true));
    }

    #[cfg(unix)]
    #[test]
    fn a_stderr_read_error_interrupts_a_blocked_stdout_join_after_parent_exit() {
        struct FailingReader;

        impl Read for FailingReader {
            fn read(&mut self, _buffer: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("forced stderr read error"))
            }
        }

        let temp = tempfile::tempdir().expect("tempdir");
        let shell_pid_path = temp.path().join("shell.pid");
        let child_pid_path = temp.path().join("child.pid");
        let script = process_group_script(&shell_pid_path, &child_pid_path, false);
        let reader_shell_pid_path = shell_pid_path.clone();
        let reader_child_pid_path = child_pid_path.clone();
        let (result_sender, result_receiver) = std::sync::mpsc::channel();
        let command_thread = thread::spawn(move || {
            let context = read_only_context(Duration::from_secs(2));
            let mut reader_index = 0;
            let result = run_bounded_command_with(
                &context,
                BoundedCommand::new("/bin/sh").args(["-c", script.as_str()]),
                |child| child.try_wait(),
                move |reader: Box<dyn Read + Send>, limit| {
                    let index = reader_index;
                    reader_index += 1;
                    let shell_pid_path = reader_shell_pid_path.clone();
                    let child_pid_path = reader_child_pid_path.clone();
                    thread::Builder::new()
                        .spawn(move || {
                            if index == 1 {
                                wait_for_pid(&shell_pid_path);
                                wait_for_pid(&child_pid_path);
                                drop(reader);
                                read_capped(FailingReader, limit)
                            } else {
                                read_capped(reader, limit)
                            }
                        })
                        .map_err(|error| io::Error::other(error.to_string()))
                },
            );
            let _ = result_sender.send(matches!(result, Err(ServiceError::Internal)));
        });

        let shell_pid = wait_for_pid(&shell_pid_path);
        let child_pid = wait_for_pid(&child_pid_path);
        let completed_before_cleanup = result_receiver
            .recv_timeout(Duration::from_millis(300))
            .unwrap_or(false);
        kill_process_group_for_test(shell_pid);
        command_thread.join().expect("command thread");
        let processes_reaped = wait_for_processes_to_exit(shell_pid, child_pid);

        assert_eq!((completed_before_cleanup, processes_reaped), (true, true));
    }

    #[cfg(unix)]
    #[test]
    fn a_reader_panic_terminates_the_process_group_and_joins_the_other_reader() {
        let temp = tempfile::tempdir().expect("tempdir");
        let shell_pid_path = temp.path().join("shell.pid");
        let child_pid_path = temp.path().join("child.pid");
        let script = process_group_script(&shell_pid_path, &child_pid_path, false);
        let readers_finished = Arc::new(AtomicUsize::new(0));
        let mut reader_index = 0;
        let context = read_only_context(Duration::from_secs(2));
        let panic_shell_pid_path = shell_pid_path.clone();
        let panic_child_pid_path = child_pid_path.clone();

        let result = run_bounded_command_with(
            &context,
            BoundedCommand::new("/bin/sh").args(["-c", script.as_str()]),
            |child| child.try_wait(),
            {
                let readers_finished = readers_finished.clone();
                move |reader: Box<dyn Read + Send>, limit| {
                    let index = reader_index;
                    reader_index += 1;
                    let readers_finished = readers_finished.clone();
                    let shell_pid_path = panic_shell_pid_path.clone();
                    let child_pid_path = panic_child_pid_path.clone();
                    thread::Builder::new()
                        .spawn(move || {
                            if index == 0 {
                                wait_for_pid(&shell_pid_path);
                                wait_for_pid(&child_pid_path);
                                drop(reader);
                                readers_finished.fetch_add(1, Ordering::SeqCst);
                                panic!("forced reader panic");
                            }
                            let result = read_capped(reader, limit);
                            thread::sleep(Duration::from_millis(50));
                            readers_finished.fetch_add(1, Ordering::SeqCst);
                            result
                        })
                        .map_err(|error| io::Error::other(error.to_string()))
                }
            },
        );

        let shell_pid = wait_for_pid(&shell_pid_path);
        let child_pid = wait_for_pid(&child_pid_path);
        let processes_reaped = wait_for_processes_to_exit(shell_pid, child_pid);
        let observed = (
            matches!(result, Err(ServiceError::Internal)),
            readers_finished.load(Ordering::SeqCst),
            processes_reaped,
        );
        kill_process_group_for_test(shell_pid);

        assert_eq!(observed, (true, 2, true));
    }

    #[cfg(unix)]
    #[test]
    fn a_hard_deadline_terminates_and_reaps_the_process_group() {
        use std::path::Path;
        use std::time::Instant;

        let temp = tempfile::tempdir().expect("tempdir");
        let shell_pid_path = temp.path().join("shell.pid");
        let child_pid_path = temp.path().join("child.pid");
        let script = format!(
            "echo $$ > {}; sleep 60 & echo $! > {}; wait",
            shell_pid_path.display(),
            child_pid_path.display()
        );
        let lane = LongRunningLane::spawn().expect("spawn lane");
        let reply = lane
            .handle()
            .try_submit(
                JobOptions::read_only(Duration::from_millis(250)),
                move |context| {
                    run_bounded_command(
                        context,
                        BoundedCommand::new("/bin/sh")
                            .args(["-c", script.as_str()])
                            .stdout_limit(1024)
                            .stderr_limit(1024),
                    )?;
                    Ok(json!(true))
                },
            )
            .expect("submit");

        assert_eq!(
            reply.blocking_recv().expect("reply"),
            Err(ServiceError::Timeout)
        );
        let shell_pid = read_pid(&shell_pid_path);
        let child_pid = read_pid(&child_pid_path);
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline && (process_exists(shell_pid) || process_exists(child_pid))
        {
            thread::sleep(Duration::from_millis(10));
        }
        assert!(
            !process_exists(shell_pid),
            "shell process {shell_pid} survived"
        );
        assert!(
            !process_exists(child_pid),
            "child process {child_pid} survived"
        );
        lane.shutdown().expect("shutdown");

        fn read_pid(path: &Path) -> i32 {
            std::fs::read_to_string(path)
                .expect("pid file")
                .trim()
                .parse()
                .expect("pid")
        }

        fn process_exists(pid: i32) -> bool {
            // SAFETY: signal 0 performs existence/permission checking only.
            let result = unsafe { libc::kill(pid, 0) };
            result == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_hard_deadline_terminates_a_background_child_during_reader_drain() {
        let temp = tempfile::tempdir().expect("tempdir");
        let shell_pid_path = temp.path().join("shell.pid");
        let child_pid_path = temp.path().join("child.pid");
        let script = process_group_script(&shell_pid_path, &child_pid_path, false);
        let (result_sender, result_receiver) = std::sync::mpsc::channel();
        let command_thread = thread::spawn(move || {
            let context = read_only_context(Duration::from_millis(250));
            let result = run_bounded_command(
                &context,
                BoundedCommand::new("/bin/sh").args(["-c", script.as_str()]),
            );
            let _ = result_sender.send(matches!(result, Err(ServiceError::Timeout)));
        });

        let shell_pid = wait_for_pid(&shell_pid_path);
        let child_pid = wait_for_pid(&child_pid_path);
        let result_before_cleanup = result_receiver.recv_timeout(Duration::from_millis(750));
        let completed_before_cleanup = result_before_cleanup.is_ok();
        let returned_timeout = result_before_cleanup.unwrap_or(false);
        let processes_reaped =
            completed_before_cleanup && wait_for_processes_to_exit(shell_pid, child_pid);
        kill_process_group_for_test(shell_pid);
        command_thread.join().expect("command thread");

        assert_eq!(
            (completed_before_cleanup, returned_timeout, processes_reaped),
            (true, true, true)
        );
    }

    #[test]
    fn command_output_is_capped_while_the_pipes_are_fully_drained() {
        let lane = LongRunningLane::spawn().expect("spawn lane");
        let reply = lane
            .handle()
            .try_submit(JobOptions::read_only(Duration::from_secs(2)), |context| {
                let output = run_bounded_command(
                    context,
                    BoundedCommand::new("/bin/sh")
                        .args([
                            "-c",
                            "yes stdout | head -c 65536; yes stderr | head -c 65536 >&2",
                        ])
                        .stdout_limit(4096)
                        .stderr_limit(2048),
                )?;
                Ok(json!({
                    "stdout": output.stdout.len(),
                    "stderr": output.stderr.len(),
                    "stdoutTruncated": output.stdout_truncated,
                    "stderrTruncated": output.stderr_truncated,
                }))
            })
            .expect("submit");

        assert_eq!(
            reply.blocking_recv().expect("reply").expect("command"),
            json!({
                "stdout": 4096,
                "stderr": 2048,
                "stdoutTruncated": true,
                "stderrTruncated": true,
            })
        );
        lane.shutdown().expect("shutdown");
    }

    #[cfg(unix)]
    fn read_only_context(timeout: Duration) -> JobContext {
        JobContext {
            deadline: Instant::now() + timeout,
            cancellation: CancellationToken::new(),
            lane_shutdown: CancellationToken::new(),
            ignore_caller_cancellation: false,
        }
    }

    #[cfg(unix)]
    fn process_group_script(
        shell_pid_path: &std::path::Path,
        child_pid_path: &std::path::Path,
        wait_for_child: bool,
    ) -> String {
        let tail = if wait_for_child { "wait" } else { "exit 0" };
        format!(
            "echo $$ > {}; sleep 60 & echo $! > {}; {tail}",
            shell_pid_path.display(),
            child_pid_path.display()
        )
    }

    #[cfg(unix)]
    fn wait_for_pid(path: &std::path::Path) -> i32 {
        let deadline = Instant::now() + Duration::from_secs(2);
        while Instant::now() < deadline {
            if let Ok(contents) = std::fs::read_to_string(path)
                && let Ok(pid) = contents.trim().parse()
            {
                return pid;
            }
            thread::sleep(Duration::from_millis(5));
        }
        panic!("pid file was not written: {}", path.display());
    }

    #[cfg(unix)]
    fn wait_for_processes_to_exit(shell_pid: i32, child_pid: i32) -> bool {
        let deadline = Instant::now() + Duration::from_secs(1);
        while Instant::now() < deadline {
            if !process_exists(shell_pid) && !process_exists(child_pid) {
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        false
    }

    #[cfg(unix)]
    fn process_exists(pid: i32) -> bool {
        // SAFETY: signal 0 performs existence/permission checking only.
        let result = unsafe { libc::kill(pid, 0) };
        result == 0 || io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
    }

    #[cfg(unix)]
    fn kill_process_group_for_test(group_id: i32) {
        // SAFETY: the pid came from the shell spawned into its own process group.
        let _ = unsafe { libc::kill(-group_id, libc::SIGKILL) };
    }
}
