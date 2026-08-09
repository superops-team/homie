## ADDED Requirements

### Requirement: Runtime paths are absolute and owner-only
The system SHALL derive daemon socket, lock, and logs from an absolute data directory under `<data-dir>/runtime`. The runtime directory MUST be mode 0700 and the socket and lock MUST be mode 0600.

#### Scenario: Runtime paths are created safely
- **WHEN** launcher or daemon receives an absolute data directory
- **THEN** it creates or validates owner-only runtime paths without reading environment overrides

#### Scenario: Unsafe runtime directory is rejected
- **WHEN** the runtime directory is writable by another user or resolves to an invalid ownership boundary
- **THEN** daemon startup fails with a safe error before binding a socket

### Requirement: One daemon owns each data directory
The system SHALL use a non-blocking singleton lock so only one daemon owns a data directory. A daemon MUST hold the lock for its full lifetime and MUST acquire it before deleting a stale socket.

#### Scenario: Concurrent launcher attempts converge
- **WHEN** multiple launchers start daemon processes for the same data directory
- **THEN** exactly one daemon acquires the lock and the other processes exit without altering the live socket

#### Scenario: Stale socket is removed by owner
- **WHEN** no lock owner exists and the socket is not connectable
- **THEN** the new lock owner removes the stale socket and binds the endpoint

### Requirement: Launcher is explicit and absolute-path based
The system SHALL expose `RuntimeLauncher::ensure_running` separately from client connect. Launcher input MUST contain an absolute data directory and an absolute daemon executable path, and the launcher MUST NOT use PATH or environment variables to resolve either.

#### Scenario: Live daemon is not restarted
- **WHEN** launcher hello probe succeeds
- **THEN** launcher returns the live endpoint without spawning or comparing hashes

#### Scenario: Missing daemon is started
- **WHEN** launcher probe reports an unavailable endpoint
- **THEN** launcher starts the absolute daemon executable in a detached process group, appends output to the fixed boot log, and returns without waiting for ready

#### Scenario: Invalid executable path fails
- **WHEN** the daemon executable is relative, missing, or not executable
- **THEN** launcher returns a stable launch error and does not invoke a shell

#### Scenario: Live endpoint is incompatible
- **WHEN** endpoint responds with version mismatch, unauthorized, protocol error, or a different executable hash
- **THEN** launcher returns that error without spawning, replacing, or restarting the live daemon

### Requirement: RuntimeActor owns blocking runtime state
The daemon SHALL move the production `RuntimeSupervisor`, SQLite connection, live registry, and event mutation source into one `RuntimeActor` running on a dedicated OS thread named `homie-runtime-actor`. Socket tasks MUST access them only through a capacity-256 command channel and oneshot replies. Tokio async workers MUST NOT execute SQLite, PTY, git, history, or blocking file operations.

#### Scenario: Runtime mutation is serialized
- **WHEN** concurrent connections submit runtime mutations
- **THEN** the actor executes each accepted mutation with a single runtime owner and returns correlated replies

#### Scenario: Actor-owned blocking runtime call executes
- **WHEN** a handler performs SQLite, PTY, live registry, or runtime mutation work
- **THEN** the work runs on the dedicated actor thread rather than a Tokio async worker

#### Scenario: Actor queue is full
- **WHEN** the actor command queue reaches 256 pending commands
- **THEN** the server returns `backpressure` without blocking a Tokio socket worker

#### Scenario: Test backend is injected internally
- **WHEN** server integration tests construct the server library
- **THEN** they can inject a deterministic backend without adding any daemon binary test-mode option

### Requirement: Long-running operations use one bounded lane
The daemon SHALL execute git, history, and bounded output scans on one dedicated OS worker named `homie-runtime-long-running` with a 32-job queue. The lane MUST receive only owned path/DTO snapshots and MUST NOT own or call `Storage`, `RuntimeSupervisor`, or live registry.

#### Scenario: Long operation executes
- **WHEN** dispatcher accepts a git, history, artifact, status, or snapshot operation
- **THEN** it uses actor prepare, lane execute, and actor commit without running the long operation on RuntimeActor or a Tokio worker

#### Scenario: Long-running queue is full
- **WHEN** 32 jobs are pending and a 33rd job is submitted
- **THEN** the daemon returns `backpressure` and does not start that job

#### Scenario: Queued job expires or is cancelled
- **WHEN** a queued job reaches its deadline or its waiter is removed before execution
- **THEN** the lane skips it and actor commit does not occur

#### Scenario: Git hard deadline expires
- **WHEN** a git child exceeds the fixed method deadline
- **THEN** the daemon terminates and reaps the child process group and returns `timeout`

#### Scenario: Started worktree mutation caller is cancelled
- **WHEN** caller cancellation occurs after worktree create/remove starts
- **THEN** the mutation continues to success or its 60-second hard deadline and is never automatically replayed

#### Scenario: Long-running jobs are serialized
- **WHEN** jobs for one or multiple repositories are accepted
- **THEN** the single lane worker executes them serially without a repo-key coordinator

### Requirement: Daemon startup is ordered
The daemon SHALL start in this order: validate paths, acquire lock, remove stale socket, open/migrate storage and adopt runtime facts, start actor, bind UDS, publish ready.

#### Scenario: Startup succeeds
- **WHEN** all startup stages complete
- **THEN** the daemon accepts Hello only after the production actor is ready

#### Scenario: Startup fails before ready
- **WHEN** migration, adoption, actor initialization, or bind fails
- **THEN** the daemon removes only its own socket, releases its lock, preserves session/holder/output/database data, and exits non-zero

### Requirement: Local peers are authenticated by UID
The daemon SHALL verify that every accepted UDS peer UID matches the daemon UID before processing the protocol. It SHALL permit at most 64 active client connections.

#### Scenario: Same-user peer connects
- **WHEN** the UDS peer UID equals the daemon UID
- **THEN** the daemon permits preface and Hello processing

#### Scenario: Different-user peer connects
- **WHEN** the UDS peer UID differs from the daemon UID
- **THEN** the daemon closes the connection and records only a safe `unauthorized` event

#### Scenario: Connection limit is reached
- **WHEN** a 65th client connects while 64 clients are active
- **THEN** the daemon rejects it before reading a frame payload

### Requirement: Graceful shutdown drains in a fixed order
The daemon SHALL implement `daemon.prepare_shutdown` and `daemon.shutdown`. Prepare MUST reject new mutations and flush/checkpoint durable facts. Shutdown MUST acknowledge before closing listener, streams, and actor.

#### Scenario: Administrative shutdown
- **WHEN** an authorized client calls prepare then shutdown
- **THEN** accepted responses finish, durable facts flush, shutdown ACK is sent, and the daemon exits cleanly

#### Scenario: Signal shutdown
- **WHEN** the daemon receives SIGTERM or SIGINT
- **THEN** it uses the same drain path as administrative shutdown

#### Scenario: Holder session exists during shutdown
- **WHEN** graceful daemon shutdown occurs while a holder owns a session
- **THEN** the daemon does not terminate that holder-owned process

### Requirement: Daemon restart recovers durable runtime facts
The daemon SHALL reopen storage, holder status, and output log after a hard process failure without trusting a stale in-memory state.

#### Scenario: Daemon restarts after hard exit
- **WHEN** a new singleton owner starts after the prior daemon terminates
- **THEN** it rebuilds runtime state from durable evidence and exposes a new daemon instance ID

#### Scenario: Holder liveness is uncertain
- **WHEN** durable holder evidence cannot prove a session is running
- **THEN** the daemon preserves the current detached semantics and does not mark the session running
