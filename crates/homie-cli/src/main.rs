use anyhow::Context;
use clap::{Parser, Subcommand};
use homie_agents::{AgentCatalog, HookEvent, load_manifest, parse_claude_hook, parse_codex_notify};
use homie_client::{ClientError, ClientOptions, HomieClient, LauncherOptions, RuntimeLauncher};
use homie_proto::model::{ArtifactKind, SessionSnapshot, SessionSummary};
use homie_proto::paths::RuntimeEndpoint;
use homie_proto::transport::ClientRole;
use homie_proto::{ErrorEnvelope, SessionDiffBase};
use homie_storage::{StorageConfig, UsageQuery, open_or_create};
use serde::Serialize;
use serde_json::{Value, json};
use std::io::{BufRead as _, Read as _, Write as _};
use std::path::PathBuf;
use std::time::Duration;

const MAX_CONTROL_MESSAGE_BYTES: usize = 4 * 1024 * 1024;
const STARTUP_PROBE_TIMEOUT: Duration = Duration::from_millis(250);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Parser, Debug)]
#[command(name = "homie")]
#[command(about = "Homie local development and diagnostics CLI")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand, Debug)]
enum Command {
    Doctor(DoctorArgs),
    Runtime(RuntimeCommand),
    Session(SessionCommand),
    Host(HostCommand),
    Worktree(WorktreeCommand),
    Ports(PortsArgs),
    Agent(AgentCommand),
    Usage(UsageCommand),
    Events(EventsCommand),
    ControlStdio(ControlStdioArgs),
    Hook(HookArgs),
    Notify(NotifyArgs),
    McpTools,
    McpCall(McpCallArgs),
    McpStdio(McpStdioArgs),
}

#[derive(Parser, Debug)]
struct DoctorArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct RuntimeCommand {
    #[command(subcommand)]
    command: RuntimeSubcommand,
}

#[derive(Subcommand, Debug)]
enum RuntimeSubcommand {
    Status(CommonArgs),
}

#[derive(Parser, Debug)]
struct SessionCommand {
    #[command(subcommand)]
    command: SessionSubcommand,
}

#[derive(Subcommand, Debug)]
enum SessionSubcommand {
    Create(SessionCreateArgs),
    List(CommonArgs),
    Snapshot(SessionSnapshotArgs),
    Kill(SessionKillArgs),
    Diff(SessionDiffArgs),
    History(SessionHistoryArgs),
    ResumeHistory(SessionResumeHistoryArgs),
}

#[derive(Parser, Debug)]
struct HostCommand {
    #[command(subcommand)]
    command: HostSubcommand,
}

#[derive(Subcommand, Debug)]
enum HostSubcommand {
    LocateRepo(HostLocateRepoArgs),
}

#[derive(Parser, Debug)]
struct HostLocateRepoArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    origin_url: Option<String>,
    #[arg(long)]
    cwd: Option<PathBuf>,
    #[arg(long)]
    session_id: Option<String>,
    #[arg(long = "candidate")]
    candidates: Vec<PathBuf>,
}

#[derive(Parser, Debug)]
struct WorktreeCommand {
    #[command(subcommand)]
    command: WorktreeSubcommand,
}

#[derive(Subcommand, Debug)]
enum WorktreeSubcommand {
    List(WorktreeListArgs),
    Create(WorktreeCreateArgs),
    Remove(WorktreeRemoveArgs),
}

#[derive(Parser, Debug)]
struct WorktreeListArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    repo: PathBuf,
    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct WorktreeCreateArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    repo: PathBuf,
    #[arg(long)]
    branch: Option<String>,
    #[arg(long)]
    base: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct WorktreeRemoveArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    repo: PathBuf,
    #[arg(long)]
    path: PathBuf,
    #[arg(long)]
    force: bool,
    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct PortsArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct AgentCommand {
    #[command(subcommand)]
    command: AgentSubcommand,
}

#[derive(Subcommand, Debug)]
enum AgentSubcommand {
    Readiness(AgentReadinessArgs),
}

#[derive(Parser, Debug)]
struct AgentReadinessArgs {
    #[arg(long)]
    descriptor_dir: PathBuf,
    #[arg(long)]
    bin_dir: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct UsageCommand {
    #[command(subcommand)]
    command: UsageSubcommand,
}

#[derive(Subcommand, Debug)]
enum UsageSubcommand {
    Summary(UsageSummaryArgs),
}

#[derive(Parser, Debug)]
struct UsageSummaryArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    session_id: Option<String>,
    #[arg(long)]
    provider_id: Option<String>,
    #[arg(long)]
    model: Option<String>,
    #[arg(long)]
    from: Option<i64>,
    #[arg(long)]
    to: Option<i64>,
    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct EventsCommand {
    #[command(subcommand)]
    command: EventsSubcommand,
}

#[derive(Subcommand, Debug)]
enum EventsSubcommand {
    List(EventsListArgs),
    Wait(EventsListArgs),
}

#[derive(Parser, Debug)]
struct EventsListArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long, default_value_t = 0)]
    after_seq: u64,
    #[arg(long = "event")]
    event_filter: Vec<String>,
    #[arg(long, default_value_t = 30_000)]
    timeout_ms: u64,
}

#[derive(Parser, Debug)]
struct ControlStdioArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
}

#[derive(Parser, Debug)]
struct SessionCreateArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    workspace: PathBuf,
    #[arg(long)]
    title: Option<String>,
    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct SessionSnapshotArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    id: String,
    #[arg(long, default_value_t = 0)]
    offset: u64,
    #[arg(long, default_value_t = 8192)]
    max_bytes: usize,
}

#[derive(Parser, Debug)]
struct SessionKillArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    id: String,
}

#[derive(Parser, Debug)]
struct SessionDiffArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    id: String,
    #[arg(long, default_value = "default-branch")]
    base: String,
}

#[derive(Parser, Debug)]
struct SessionHistoryArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    claude_root: PathBuf,
    #[arg(long)]
    codex_root: PathBuf,
    #[arg(long = "tracked")]
    tracked: Vec<String>,
}

#[derive(Parser, Debug)]
struct SessionResumeHistoryArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    agent_kind: String,
    #[arg(long)]
    external_id: String,
    #[arg(long)]
    cwd: PathBuf,
    #[arg(long)]
    title: Option<String>,
}

#[derive(Parser, Debug)]
struct CommonArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct HookArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    event: String,
    payload: Option<String>,
}

#[derive(Parser, Debug)]
struct NotifyArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(trailing_var_arg = true)]
    args: Vec<String>,
}

#[derive(Parser, Debug)]
struct McpCallArgs {
    #[arg(long)]
    tool: String,
}

#[derive(Parser, Debug)]
struct McpStdioArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    session_id: Option<String>,
    #[arg(long)]
    parent_session_id: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DoctorOutput {
    status: String,
    database_path: String,
    schema_version: i64,
    foreign_keys: bool,
    journal_mode: String,
    default_agent_profile: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeStatusOutput {
    status: String,
    runtime_process: String,
    daemon_pid: u32,
    daemon_instance_id: String,
    daemon_version: String,
    method_capabilities: Vec<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Some(Command::Doctor(args)) => doctor(args),
        Some(Command::Runtime(command)) => match command.command {
            RuntimeSubcommand::Status(args) => runtime_status(args).await,
        },
        Some(Command::Session(command)) => match command.command {
            SessionSubcommand::Create(args) => session_create(args).await,
            SessionSubcommand::List(args) => session_list(args).await,
            SessionSubcommand::Snapshot(args) => session_snapshot(args).await,
            SessionSubcommand::Kill(args) => session_kill(args).await,
            SessionSubcommand::Diff(args) => session_diff(args).await,
            SessionSubcommand::History(args) => session_history(args).await,
            SessionSubcommand::ResumeHistory(args) => session_resume_history(args).await,
        },
        Some(Command::Host(command)) => match command.command {
            HostSubcommand::LocateRepo(args) => host_locate_repo(args).await,
        },
        Some(Command::Worktree(command)) => match command.command {
            WorktreeSubcommand::List(args) => worktree_list(args).await,
            WorktreeSubcommand::Create(args) => worktree_create(args).await,
            WorktreeSubcommand::Remove(args) => worktree_remove(args).await,
        },
        Some(Command::Ports(args)) => ports(args).await,
        Some(Command::Agent(command)) => match command.command {
            AgentSubcommand::Readiness(args) => agent_readiness(args),
        },
        Some(Command::Usage(command)) => match command.command {
            UsageSubcommand::Summary(args) => usage_summary(args),
        },
        Some(Command::Events(command)) => match command.command {
            EventsSubcommand::List(args) => events_list(args).await,
            EventsSubcommand::Wait(args) => events_wait(args).await,
        },
        Some(Command::ControlStdio(args)) => control_stdio(args).await,
        Some(Command::Hook(args)) => hook(args).await,
        Some(Command::Notify(args)) => notify(args).await,
        Some(Command::McpTools) => mcp_tools(),
        Some(Command::McpCall(args)) => mcp_call(args),
        Some(Command::McpStdio(args)) => mcp_stdio(args).await,
        None => app_launch(),
    }
}

fn doctor(args: DoctorArgs) -> anyhow::Result<()> {
    let data_dir = match args.data_dir {
        Some(data_dir) => data_dir,
        None => default_data_dir()?,
    };
    let storage = open_or_create(StorageConfig {
        data_dir: data_dir.clone(),
    })
    .with_context(|| format!("open storage at {}", data_dir.display()))?;
    storage.migrate().context("migrate storage")?;
    storage.seed_defaults().context("seed default config")?;
    let health = storage.health_check().context("check storage health")?;
    let output = DoctorOutput {
        status: "ok".to_string(),
        database_path: health.database_path.display().to_string(),
        schema_version: health.schema_version,
        foreign_keys: health.foreign_keys,
        journal_mode: health.journal_mode,
        default_agent_profile: Some("agent_codex_default".to_string()),
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Homie doctor: {}", output.status);
        println!("database: {}", output.database_path);
        println!("schema version: {}", output.schema_version);
        println!("foreign keys: {}", output.foreign_keys);
        println!("journal mode: {}", output.journal_mode);
        println!(
            "default agent profile: {}",
            output.default_agent_profile.as_deref().unwrap_or("none")
        );
    }
    Ok(())
}

async fn runtime_status(args: CommonArgs) -> anyhow::Result<()> {
    let client = connect_runtime_client(args.data_dir, ClientRole::Cli).await?;
    let hello = client
        .hello()
        .context("connected runtime did not provide Hello metadata")?;
    let output = RuntimeStatusOutput {
        status: "ready".to_string(),
        runtime_process: "running".to_string(),
        daemon_pid: hello.daemon_pid,
        daemon_instance_id: hello.daemon_instance_id,
        daemon_version: hello.daemon_version,
        method_capabilities: hello.method_capabilities,
    };
    if args.json {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("runtime: {}", output.runtime_process);
        println!("daemon pid: {}", output.daemon_pid);
        println!("daemon instance: {}", output.daemon_instance_id);
        println!("daemon version: {}", output.daemon_version);
    }
    Ok(())
}

async fn session_create(args: SessionCreateArgs) -> anyhow::Result<()> {
    let client = connect_runtime_client(args.data_dir, ClientRole::Cli).await?;
    let session = client
        .spawn_shell(&args.workspace, args.title.as_deref())
        .await
        .context("spawn session")?;
    print_session_or_json(&session, args.json)
}

async fn session_list(args: CommonArgs) -> anyhow::Result<()> {
    let client = connect_runtime_client(args.data_dir, ClientRole::Cli).await?;
    let sessions = client.list_sessions().await.context("list sessions")?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&sessions)?);
    } else {
        for session in sessions {
            println!(
                "{}\t{}\t{}\t{}",
                session.id, session.runtime_id, session.status, session.title
            );
        }
    }
    Ok(())
}

async fn session_snapshot(args: SessionSnapshotArgs) -> anyhow::Result<()> {
    let client = connect_runtime_client(args.data_dir, ClientRole::Cli).await?;
    let snapshot = client
        .session_snapshot(&args.id, args.offset, args.max_bytes)
        .await
        .context("read session snapshot")?;
    let output = session_snapshot_json(snapshot);
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

async fn session_kill(args: SessionKillArgs) -> anyhow::Result<()> {
    let client = connect_runtime_client(args.data_dir, ClientRole::Cli).await?;
    client
        .terminate_session(&args.id)
        .await
        .context("kill session")?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "ok": true,
            "sessionId": args.id
        }))?
    );
    Ok(())
}

async fn session_diff(args: SessionDiffArgs) -> anyhow::Result<()> {
    let client = connect_runtime_client(args.data_dir, ClientRole::Cli).await?;
    let base = parse_diff_base(&args.base)?;
    let diff = client
        .read_diff(&args.id, base)
        .await
        .context("read session diff")?;
    let patch_text = String::from_utf8_lossy(&diff.patch).to_string();
    let summary = summarize_unified_diff(&patch_text);
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "sessionId": args.id,
            "repoRoot": diff.repo_root,
            "baseRef": diff.base_ref,
            "truncated": diff.truncated,
            "files": summary.files,
            "additions": summary.additions,
            "deletions": summary.deletions,
            "patchText": patch_text
        }))?
    );
    Ok(())
}

fn parse_diff_base(value: &str) -> anyhow::Result<SessionDiffBase> {
    match value {
        "default-branch" | "defaultBranch" | "default" => Ok(SessionDiffBase::DefaultBranch),
        "head" | "HEAD" => Ok(SessionDiffBase::Head),
        other => anyhow::bail!("unsupported diff base: {other}"),
    }
}

async fn session_history(args: SessionHistoryArgs) -> anyhow::Result<()> {
    let client = connect_runtime_client(args.data_dir, ClientRole::Cli).await?;
    let output = client
        .session_history(homie_proto::SessionHistoryRequest {
            claude_root: args.claude_root.display().to_string(),
            codex_root: args.codex_root.display().to_string(),
            tracked: args.tracked,
        })
        .await
        .context("scan session history")?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

async fn session_resume_history(args: SessionResumeHistoryArgs) -> anyhow::Result<()> {
    let client = connect_runtime_client(args.data_dir, ClientRole::Cli).await?;
    let output = client
        .resume_from_history(homie_proto::SessionResumeFromHistoryRequest {
            agent_kind: args.agent_kind,
            external_id: args.external_id,
            cwd: args.cwd.display().to_string(),
            title: args.title,
        })
        .await
        .context("resume session history")?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

async fn host_locate_repo(args: HostLocateRepoArgs) -> anyhow::Result<()> {
    let output = if args.cwd.is_some() || !args.candidates.is_empty() {
        homie_remote::locate_repo(
            args.cwd.as_deref(),
            args.origin_url.as_deref(),
            &args.candidates,
        )
        .context("locate repo from local candidates")?
    } else {
        let client = connect_runtime_client(args.data_dir, ClientRole::Cli).await?;
        let output = client
            .locate_repo(homie_proto::HostLocateRepoParams {
                host: None,
                origin_url: args.origin_url,
                session_id: args.session_id.map(Into::into),
            })
            .await
            .context("locate repo through runtime")?;
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

async fn worktree_list(args: WorktreeListArgs) -> anyhow::Result<()> {
    let client = connect_runtime_client(args.data_dir, ClientRole::Cli).await?;
    let worktrees = client
        .worktree_list(homie_proto::WorktreeListRequest {
            repo_path: args.repo.display().to_string(),
        })
        .await
        .context("list worktrees")?;
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "repo": args.repo,
                "worktrees": worktrees
            }))?
        );
    } else if worktrees.is_empty() {
        println!("No worktrees for {}.", args.repo.display());
    } else {
        for worktree in worktrees {
            println!(
                "{}\t{}",
                worktree.branch.as_deref().unwrap_or("-"),
                worktree.path
            );
        }
    }
    Ok(())
}

async fn worktree_create(args: WorktreeCreateArgs) -> anyhow::Result<()> {
    let client = connect_runtime_client(args.data_dir, ClientRole::Cli).await?;
    let worktree = client
        .worktree_create(homie_proto::WorktreeCreateRequest {
            repo_path: args.repo.display().to_string(),
            branch: args.branch,
            base: args.base,
        })
        .await
        .context("create worktree")?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&worktree)?);
    } else {
        println!(
            "{}\t{}",
            worktree.branch.as_deref().unwrap_or("-"),
            worktree.path
        );
    }
    Ok(())
}

async fn worktree_remove(args: WorktreeRemoveArgs) -> anyhow::Result<()> {
    let client = connect_runtime_client(args.data_dir, ClientRole::Cli).await?;
    let path = args.path.display().to_string();
    client
        .worktree_remove(homie_proto::WorktreeRemoveRequest {
            repo_path: args.repo.display().to_string(),
            worktree_path: path.clone(),
            force: args.force,
        })
        .await
        .context("remove worktree")?;
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ok": true,
                "path": path
            }))?
        );
    } else {
        println!("removed {path}");
    }
    Ok(())
}

async fn ports(args: PortsArgs) -> anyhow::Result<()> {
    let client = connect_runtime_client(args.data_dir, ClientRole::Cli).await?;
    let rows = client.list_ports().await.context("list ports")?;
    if args.json {
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "ports": rows
            }))?
        );
    } else if rows.is_empty() {
        println!("No listening ports tracked.");
    } else {
        println!("PORT\tSESSION\tURL");
        for row in rows {
            println!("{}\t{}\t{}", row.port, row.session_title, row.url);
        }
    }
    Ok(())
}

fn agent_readiness(args: AgentReadinessArgs) -> anyhow::Result<()> {
    let catalog = AgentCatalog::new(load_agent_manifests(&args.descriptor_dir)?);
    let readiness =
        catalog.readiness_with_resolver(|binary| resolve_binary(binary, args.bin_dir.as_deref()));
    if args.json {
        println!("{}", serde_json::to_string_pretty(&readiness)?);
    } else {
        for agent in readiness.agents {
            let state = if agent.available {
                "available"
            } else {
                "missing"
            };
            println!(
                "{}\t{}\t{}",
                agent.id,
                state,
                agent.path.as_deref().unwrap_or("-")
            );
        }
    }
    Ok(())
}

fn load_agent_manifests(dir: &std::path::Path) -> anyhow::Result<Vec<homie_agents::AgentManifest>> {
    let mut paths = std::fs::read_dir(dir)
        .with_context(|| format!("read descriptor dir {}", dir.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort();
    let mut manifests = Vec::new();
    for path in paths
        .into_iter()
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
    {
        let bytes = std::fs::read(&path).with_context(|| format!("read {}", path.display()))?;
        manifests.push(load_manifest(&bytes).with_context(|| format!("load {}", path.display()))?);
    }
    Ok(manifests)
}

fn usage_summary(args: UsageSummaryArgs) -> anyhow::Result<()> {
    let storage = open_ready_storage(args.data_dir)?;
    let totals = storage
        .query_usage_totals(UsageQuery {
            session_id: args.session_id,
            provider_id: args.provider_id,
            model: args.model,
            from: args.from,
            to: args.to,
        })
        .context("query usage totals")?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&totals_json(&totals))?);
    } else {
        println!("events: {}", totals.events);
        println!("tokens: {}", totals.total_tokens);
        println!("estimated cost: {:.6}", totals.estimated_cost);
        println!("billed cost: {:.6}", totals.billed_cost);
    }
    Ok(())
}

fn totals_json(totals: &homie_storage::UsageTotals) -> Value {
    json!({
        "events": totals.events,
        "inputTokens": totals.input_tokens,
        "outputTokens": totals.output_tokens,
        "cachedInputTokens": totals.cached_input_tokens,
        "cacheReadTokens": totals.cache_read_tokens,
        "cacheWriteTokens": totals.cache_write_tokens,
        "cacheWrite5mTokens": totals.cache_write_5m_tokens,
        "cacheWrite1hTokens": totals.cache_write_1h_tokens,
        "reasoningTokens": totals.reasoning_tokens,
        "totalTokens": totals.total_tokens,
        "estimatedCost": totals.estimated_cost,
        "billedCost": totals.billed_cost,
        "authoritativeBillingAvailable": totals.authoritative_billing_available
    })
}

fn resolve_binary(binary: &str, bin_dir: Option<&std::path::Path>) -> Option<String> {
    if let Some(bin_dir) = bin_dir {
        let candidate = bin_dir.join(binary);
        return is_executable(&candidate).then(|| candidate.display().to_string());
    }
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|dir| dir.join(binary))
            .find(|candidate| is_executable(candidate))
            .map(|candidate| candidate.display().to_string())
    })
}

fn is_executable(path: &std::path::Path) -> bool {
    path.is_file()
}

fn session_snapshot_json(snapshot: SessionSnapshot) -> Value {
    json!({
        "session": snapshot.session,
        "status": {
            "status": snapshot.status.status,
            "needsInput": snapshot.status.needs_input,
            "turnCompleted": snapshot.status.turn_completed,
            "screenLines": snapshot.status.screen_lines,
            "screenObservation": snapshot.status.screen_observation.map(|observation| {
                json!({
                    "state": format!("{:?}", observation.state),
                    "matchedRuleId": observation.matched_rule_id,
                    "contentSeq": observation.content_seq
                })
            })
        },
        "outputOffset": snapshot.output_offset,
        "outputText": snapshot.output_text,
        "holder": snapshot.holder.map(|holder| {
            json!({
                "pid": holder.pid,
                "status": holder.status,
                "treeSize": holder.tree_size,
                "cols": holder.cols,
                "rows": holder.rows,
                "logOffset": holder.log_offset,
                "epochOffset": holder.epoch_offset
            })
        })
    })
}

async fn events_list(args: EventsListArgs) -> anyhow::Result<()> {
    request_events(args, 0).await
}

async fn events_wait(args: EventsListArgs) -> anyhow::Result<()> {
    let timeout_ms = args.timeout_ms;
    request_events(args, timeout_ms).await
}

async fn request_events(args: EventsListArgs, timeout_ms: u64) -> anyhow::Result<()> {
    let client = connect_runtime_client(args.data_dir, ClientRole::Cli).await?;
    let output: Value = client
        .request(
            homie_proto::Method::EVENTS_WAIT,
            homie_proto::EventsWaitRequest {
                after_seq: args.after_seq,
                timeout_ms,
                event_filter: args.event_filter,
            },
        )
        .await
        .context("wait for events")?;
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

async fn control_stdio(args: ControlStdioArgs) -> anyhow::Result<()> {
    let client = connect_runtime_client(args.data_dir, ClientRole::Cli).await?;
    let stdin = std::io::stdin();
    let mut stdin = stdin.lock();
    let mut stdout = std::io::stdout();
    loop {
        let Some(message) = read_control_message(&mut stdin)? else {
            break;
        };
        let homie_proto::ControlMessage::Request {
            request_id,
            method,
            params,
        } = message
        else {
            anyhow::bail!("control-stdio accepts request messages only");
        };
        let response = match client.request::<_, Value>(&method, params).await {
            Ok(result) => homie_proto::ControlMessage::success(request_id, result),
            Err(error) => {
                homie_proto::ControlMessage::failure(request_id, client_error_envelope(&error))
            }
        };
        serde_json::to_writer(&mut stdout, &response).context("write control response")?;
        stdout.write_all(b"\n").context("write control newline")?;
        stdout.flush().context("flush control response")?;
    }
    Ok(())
}

async fn connect_runtime_client(
    data_dir: Option<PathBuf>,
    role: ClientRole,
) -> anyhow::Result<HomieClient> {
    let data_dir = match data_dir {
        Some(data_dir) => data_dir,
        None => default_data_dir()?,
    };
    if !data_dir.is_absolute() {
        anyhow::bail!("runtime data directory must be absolute");
    }
    let daemon_executable = canonical_sibling_daemon()?;
    let paths = RuntimeLauncher::ensure_running(LauncherOptions {
        data_dir: data_dir.clone(),
        daemon_executable,
        startup_probe_timeout: STARTUP_PROBE_TIMEOUT,
    })
    .await
    .with_context(|| format!("ensure runtime daemon for {}", data_dir.display()))?;
    let endpoint = RuntimeEndpoint::new(paths.socket).context("build runtime endpoint")?;
    HomieClient::connect(ClientOptions {
        endpoint,
        role,
        connect_timeout: CONNECT_TIMEOUT,
        request_timeout: REQUEST_TIMEOUT,
    })
    .await
    .with_context(|| format!("connect runtime daemon for {}", data_dir.display()))
}

fn canonical_sibling_daemon() -> anyhow::Result<PathBuf> {
    let current_executable =
        std::fs::canonicalize(std::env::current_exe().context("resolve current executable")?)
            .context("canonicalize current executable")?;
    let parent = current_executable
        .parent()
        .context("current executable has no parent directory")?;
    std::fs::canonicalize(parent.join("homie-runtime-daemon"))
        .context("canonicalize sibling homie-runtime-daemon")
}

fn read_control_message(
    reader: &mut impl std::io::BufRead,
) -> anyhow::Result<Option<homie_proto::ControlMessage>> {
    let mut bytes = Vec::with_capacity(8 * 1024);
    let mut limited = reader.take((MAX_CONTROL_MESSAGE_BYTES + 2) as u64);
    if limited
        .read_until(b'\n', &mut bytes)
        .context("read control message")?
        == 0
    {
        return Ok(None);
    }
    let terminated = bytes.last() == Some(&b'\n');
    let message_len = bytes.len().saturating_sub(usize::from(terminated));
    if message_len > MAX_CONTROL_MESSAGE_BYTES {
        anyhow::bail!("control message exceeds 4194304 bytes");
    }
    if terminated {
        bytes.pop();
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
        }
    }
    serde_json::from_slice(&bytes)
        .map(Some)
        .context("parse control message")
}

fn client_error_envelope(error: &ClientError) -> ErrorEnvelope {
    match error {
        ClientError::Remote(envelope) => envelope.as_ref().clone(),
        _ => ErrorEnvelope::new(error.code(), "runtime request failed", false),
    }
}

#[derive(Clone, Copy)]
struct DiffSummary {
    files: usize,
    additions: usize,
    deletions: usize,
}

fn summarize_unified_diff(patch: &str) -> DiffSummary {
    let mut summary = DiffSummary {
        files: 0,
        additions: 0,
        deletions: 0,
    };
    for line in patch.lines() {
        if line.starts_with("diff --git ") {
            summary.files += 1;
        } else if line.starts_with('+') && !line.starts_with("+++ ") {
            summary.additions += 1;
        } else if line.starts_with('-') && !line.starts_with("--- ") {
            summary.deletions += 1;
        }
    }
    summary
}

fn open_ready_storage(data_dir: Option<PathBuf>) -> anyhow::Result<homie_storage::Storage> {
    let data_dir = match data_dir {
        Some(data_dir) => data_dir,
        None => default_data_dir()?,
    };
    let storage = open_or_create(StorageConfig {
        data_dir: data_dir.clone(),
    })
    .with_context(|| format!("open storage at {}", data_dir.display()))?;
    storage.migrate().context("migrate storage")?;
    storage.seed_defaults().context("seed default config")?;
    Ok(storage)
}

fn print_session_or_json(session: &SessionSummary, json: bool) -> anyhow::Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(session)?);
    } else {
        println!(
            "{}\t{}\t{}\t{}",
            session.id, session.runtime_id, session.status, session.title
        );
    }
    Ok(())
}

async fn hook(args: HookArgs) -> anyhow::Result<()> {
    let output = hook_output(&args.event, args.payload.as_deref())?;
    if let Some(data_dir) = args.data_dir
        && let (Some(session_id), Some(needs_input)) = (
            output.get("sessionId").and_then(Value::as_str),
            output.get("needsInput").cloned(),
        )
    {
        let detail = serde_json::from_value(needs_input).context("decode needsInput")?;
        let client = connect_runtime_client(Some(data_dir), ClientRole::Cli).await?;
        client
            .report_needs_input(session_id, &detail)
            .await
            .context("persist hook report")?;
    }
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

async fn notify(args: NotifyArgs) -> anyhow::Result<()> {
    let joined = (!args.args.is_empty()).then(|| args.args.join(" "));
    let output = notify_output(joined.as_deref())?;
    if let Some(data_dir) = args.data_dir
        && output["event"]["kind"] == "codexTurnComplete"
        && let Some(session_id) = output.get("sessionId").and_then(Value::as_str)
    {
        let client = connect_runtime_client(Some(data_dir), ClientRole::Cli).await?;
        client
            .report_turn_complete(session_id)
            .await
            .context("persist notify report")?;
    }
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn hook_output(event: &str, payload_json: Option<&str>) -> anyhow::Result<Value> {
    let payload = parse_payload(payload_json)?;
    if let Some(parsed) = parse_claude_hook(event, &payload) {
        if let HookEvent::Unknown(value) = &parsed.event {
            return Ok(json!({
                "event": { "kind": "unknown", "value": value },
                "sessionId": parsed.session_id,
                "transcriptPath": parsed.transcript_path,
                "safeSummary": parsed.safe_summary
            }));
        }
        return serde_json::to_value(parsed).context("serialize hook result");
    }
    Ok(json!({
        "event": { "kind": "unknown", "value": event },
        "sessionId": payload.get("session_id").and_then(Value::as_str),
        "safeSummary": "{}"
    }))
}

fn notify_output(payload_json: Option<&str>) -> anyhow::Result<Value> {
    let payload = parse_payload(payload_json)?;
    if let Some(parsed) = parse_codex_notify(&payload) {
        return serde_json::to_value(parsed).context("serialize notify result");
    }
    Ok(json!({
        "event": { "kind": "unknown", "value": payload.get("type").and_then(Value::as_str).unwrap_or("unknown") },
        "sessionId": payload.get("thread-id").and_then(Value::as_str),
        "safeSummary": "{}"
    }))
}

fn parse_payload(input: Option<&str>) -> anyhow::Result<Value> {
    let Some(input) = input.filter(|value| !value.trim().is_empty()) else {
        return Ok(Value::Object(Default::default()));
    };
    serde_json::from_str(input).with_context(|| "parse hook/notify JSON payload")
}

fn mcp_tools() -> anyhow::Result<()> {
    let context = McpRuntimeContext::no_runtime();
    let tools = json!({ "tools": mcp_tool_names(&context) });
    println!("{}", serde_json::to_string_pretty(&tools)?);
    Ok(())
}

fn mcp_call(args: McpCallArgs) -> anyhow::Result<()> {
    let result = match args.tool.as_str() {
        "list_agents" => json!({ "ok": [] }),
        "whoami" => json!({ "ok": { "sessionId": null, "title": "unbound" } }),
        other => json!({ "error": format!("unsupported tool: {other}") }),
    };
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

async fn mcp_stdio(args: McpStdioArgs) -> anyhow::Result<()> {
    let context = McpRuntimeContext::open(args).await?;
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = mcp_stdio_response_with_context(&line, &context).await;
        println!("{}", serde_json::to_string(&response)?);
    }
    Ok(())
}

#[cfg(test)]
async fn mcp_stdio_response(line: &str) -> Value {
    mcp_stdio_response_with_context(line, &McpRuntimeContext::no_runtime()).await
}

struct McpRuntimeContext {
    client: Option<HomieClient>,
    session_id: Option<String>,
    parent_session_id: Option<String>,
}

impl McpRuntimeContext {
    async fn open(args: McpStdioArgs) -> anyhow::Result<Self> {
        let client = match args.data_dir {
            Some(data_dir) => Some(connect_runtime_client(Some(data_dir), ClientRole::Mcp).await?),
            None => None,
        };
        Ok(Self {
            client,
            session_id: args.session_id,
            parent_session_id: args.parent_session_id,
        })
    }

    fn no_runtime() -> Self {
        Self {
            client: None,
            session_id: None,
            parent_session_id: None,
        }
    }
}

#[derive(Debug)]
struct McpToolError {
    code: i64,
    message: String,
}

impl McpToolError {
    fn invalid_params(message: impl Into<String>) -> Self {
        Self {
            code: -32602,
            message: message.into(),
        }
    }

    fn unsupported(tool: &str) -> Self {
        Self {
            code: -32601,
            message: format!("unsupported tool: {tool}"),
        }
    }

    fn runtime(message: impl Into<String>) -> Self {
        Self {
            code: -32000,
            message: message.into(),
        }
    }

    fn client(error: ClientError) -> Self {
        if error.code() == "method_not_found" {
            return Self {
                code: -32601,
                message: "runtime method is not available".to_string(),
            };
        }
        match error {
            ClientError::Remote(_) | ClientError::BadRequest(_) => Self::runtime(error.to_string()),
            ClientError::Timeout
            | ClientError::Unavailable
            | ClientError::Backpressure
            | ClientError::VersionMismatch
            | ClientError::Unauthorized
            | ClientError::ResyncRequired
            | ClientError::Protocol(_)
            | ClientError::Internal
            | ClientError::Json(_) => Self {
                code: -32001,
                message: "runtime transport unavailable".to_string(),
            },
        }
    }
}

async fn mcp_stdio_response_with_context(line: &str, context: &McpRuntimeContext) -> Value {
    let Ok(request) = serde_json::from_str::<Value>(line) else {
        return json!({
            "jsonrpc": "2.0",
            "id": null,
            "error": { "code": -32700, "message": "parse error" }
        });
    };
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    match request.get("method").and_then(Value::as_str) {
        Some("tools/list") => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": mcp_tool_descriptors(context)
            }
        }),
        Some("tools/call") => {
            let name = request
                .get("params")
                .and_then(|params| params.get("name"))
                .and_then(Value::as_str)
                .unwrap_or_default();
            if name.is_empty() {
                return mcp_error_response(
                    id,
                    McpToolError::invalid_params("tools/call missing name"),
                );
            }
            let arguments = request
                .get("params")
                .and_then(|params| params.get("arguments"))
                .cloned()
                .unwrap_or_else(|| json!({}));
            if !mcp_tool_is_available(context, name) {
                return mcp_error_response(id, McpToolError::unsupported(name));
            }
            match mcp_tool_payload(context, name, &arguments).await {
                Ok(payload) => mcp_text_result(id, payload),
                Err(error) => mcp_error_response(id, error),
            }
        }
        Some(other) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("unsupported method: {other}") }
        }),
        None => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32600, "message": "missing method" }
        }),
    }
}

async fn mcp_tool_payload(
    context: &McpRuntimeContext,
    name: &str,
    arguments: &Value,
) -> Result<Value, McpToolError> {
    match name {
        "list_agents" => {
            let Some(client) = &context.client else {
                return Ok(json!({ "agents": [] }));
            };
            let sessions = client.list_sessions().await.map_err(McpToolError::client)?;
            Ok(json!({ "agents": sessions }))
        }
        "whoami" => whoami_payload(context).await,
        "get_status" => {
            let client = runtime_client(context)?;
            let session_id = required_string(arguments, "sessionId")?;
            let report = client
                .status_report(&session_id)
                .await
                .map_err(McpToolError::client)?;
            Ok(json!({
                "sessionId": session_id,
                "status": report.status,
                "needsInput": report.needs_input,
                "turnCompleted": report.turn_completed,
                "screenLines": report.screen_lines,
            }))
        }
        "read_output" => {
            let client = runtime_client(context)?;
            let session_id = required_string(arguments, "sessionId")?;
            let output = client
                .read_output(&session_id)
                .await
                .map_err(McpToolError::client)?;
            Ok(json!({
                "sessionId": session_id,
                "outputText": output,
            }))
        }
        "send_prompt" => {
            let client = runtime_client(context)?;
            let session_id = required_string(arguments, "sessionId")?;
            let text = required_string(arguments, "text")?;
            let submit = arguments
                .get("submit")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            let relation = lineage_relation(context, &session_id).await?;
            if relation == "self" {
                return Err(McpToolError::runtime(format!(
                    "send_prompt cannot target the calling session ({session_id})"
                )));
            }
            let delivered = frame_lineage_message(context, &text, &relation);
            client
                .send_text(&session_id, &delivered, submit)
                .await
                .map_err(McpToolError::client)?;
            Ok(json!({
                "ok": true,
                "sessionId": session_id,
                "relation": relation,
                "attributed": delivered != text,
            }))
        }
        "spawn_agent" => {
            let client = runtime_client(context)?;
            let cwd = arguments
                .get("cwd")
                .or_else(|| arguments.get("workspace"))
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| McpToolError::invalid_params("spawn_agent requires cwd"))?;
            let title = arguments.get("title").and_then(Value::as_str);
            let session = client
                .spawn_shell_with_parent(&PathBuf::from(cwd), title, context.session_id.as_deref())
                .await
                .map_err(McpToolError::client)?;
            Ok(json!({ "session": session }))
        }
        "create_worktree" => {
            let client = runtime_client(context)?;
            let repo = required_string(arguments, "repo")?;
            let branch = arguments
                .get("branch")
                .and_then(Value::as_str)
                .map(str::to_string);
            let base = arguments
                .get("base")
                .and_then(Value::as_str)
                .map(str::to_string);
            let worktree = client
                .worktree_create(homie_proto::WorktreeCreateRequest {
                    repo_path: repo,
                    branch,
                    base,
                })
                .await
                .map_err(McpToolError::client)?;
            Ok(serde_json::to_value(worktree)
                .map_err(|error| McpToolError::runtime(error.to_string()))?)
        }
        "list_worktrees" => {
            let client = runtime_client(context)?;
            let repo = required_string(arguments, "repo")?;
            let worktrees = client
                .worktree_list(homie_proto::WorktreeListRequest {
                    repo_path: repo.clone(),
                })
                .await
                .map_err(McpToolError::client)?;
            Ok(json!({
                "repo": repo,
                "worktrees": worktrees
            }))
        }
        "remove_worktree" => {
            let client = runtime_client(context)?;
            let repo = required_string(arguments, "repo")?;
            let path = required_string(arguments, "path")?;
            let force = arguments
                .get("force")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            client
                .worktree_remove(homie_proto::WorktreeRemoveRequest {
                    repo_path: repo,
                    worktree_path: path.clone(),
                    force,
                })
                .await
                .map_err(McpToolError::client)?;
            Ok(json!({
                "ok": true,
                "path": path
            }))
        }
        "get_artifacts" => {
            let client = runtime_client(context)?;
            let session_id = required_string_any(arguments, &["session_id", "sessionId"])?;
            let scan = client
                .scan_session_artifacts(&session_id)
                .await
                .map_err(McpToolError::client)?;
            let artifacts = scan
                .artifacts
                .into_iter()
                .map(|artifact| {
                    json!({
                        "kind": artifact_kind_text(&artifact.kind),
                        "url": artifact.url,
                        "label": artifact.label,
                    })
                })
                .collect::<Vec<_>>();
            let listening_ports = scan
                .ports
                .into_iter()
                .map(|port| {
                    json!({
                        "port": port.port,
                        "url": port.url,
                    })
                })
                .collect::<Vec<_>>();
            Ok(json!({
                "sessionId": session_id,
                "artifacts": artifacts,
                "listeningPorts": listening_ports
            }))
        }
        "list_children" => {
            let client = runtime_client(context)?;
            let Some(parent) = &context.session_id else {
                return Ok(json!({
                    "children": [],
                    "count": 0,
                    "hosted": false
                }));
            };
            let children = client
                .list_child_sessions(parent)
                .await
                .map_err(McpToolError::client)?;
            let rows = children
                .into_iter()
                .map(|session| {
                    json!({
                        "id": session.id,
                        "title": session.title,
                        "status": session.status,
                        "workspace": session.workspace,
                        "parentSessionId": parent,
                        "relation": "child"
                    })
                })
                .collect::<Vec<_>>();
            Ok(json!({
                "children": rows,
                "count": rows.len()
            }))
        }
        "wait_for_agent" => wait_for_agent_payload(context, arguments).await,
        "wait_for_children" => wait_for_children_payload(context, arguments).await,
        "release_agent" => {
            let client = runtime_client(context)?;
            let session_id = required_string(arguments, "sessionId")
                .or_else(|_| required_string(arguments, "session_id"))?;
            let relation = lineage_relation(context, &session_id).await?;
            if relation == "self" {
                return Err(McpToolError::runtime(format!(
                    "release_agent cannot terminate the calling session ({session_id})"
                )));
            }
            if matches!(relation.as_str(), "parent" | "ancestor") {
                return Err(McpToolError::runtime(format!(
                    "{session_id} is the session that spawned you; releasing it would kill the conversation waiting on your result"
                )));
            }
            if relation != "child" {
                return Err(McpToolError::runtime(format!(
                    "release_agent can only release a direct child spawned by this session; {session_id} is {relation}"
                )));
            }
            client
                .terminate_session(&session_id)
                .await
                .map_err(McpToolError::client)?;
            Ok(json!({
                "ok": true,
                "sessionId": session_id
            }))
        }
        other => Err(McpToolError::unsupported(other)),
    }
}

async fn wait_for_agent_payload(
    context: &McpRuntimeContext,
    arguments: &Value,
) -> Result<Value, McpToolError> {
    let client = runtime_client(context)?;
    let session_id = required_string_any(arguments, &["session_id", "sessionId"])?;
    let mode = arguments
        .get("until")
        .and_then(Value::as_str)
        .unwrap_or("done");
    let timeout = arguments
        .get("timeout_s")
        .or_else(|| arguments.get("timeoutS"))
        .and_then(Value::as_u64)
        .unwrap_or(600);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout);
    loop {
        let report = client
            .status_report(&session_id)
            .await
            .map_err(McpToolError::client)?;
        let status = session_status_text(&report.status)?;
        let settled = child_has_reached(mode, &status);
        if settled || std::time::Instant::now() >= deadline {
            return Ok(json!({
                "settled": settled,
                "timedOut": !settled,
                "sessionId": session_id,
                "status": status,
                "needsInput": report.needs_input,
                "turnCompleted": report.turn_completed,
                "waitedFor": mode,
            }));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn wait_for_children_payload(
    context: &McpRuntimeContext,
    arguments: &Value,
) -> Result<Value, McpToolError> {
    let client = runtime_client(context)?;
    let Some(parent) = &context.session_id else {
        return Ok(json!({
            "settled": true,
            "timedOut": false,
            "children": [],
            "note": "Not running inside a Homie session"
        }));
    };
    let requested = arguments
        .get("session_ids")
        .or_else(|| arguments.get("sessionIds"))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mode = arguments
        .get("until")
        .and_then(Value::as_str)
        .unwrap_or("settled");
    let timeout = arguments
        .get("timeout_s")
        .or_else(|| arguments.get("timeoutS"))
        .and_then(Value::as_u64)
        .unwrap_or(600);
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout);
    loop {
        let mut children = client
            .list_child_sessions(parent)
            .await
            .map_err(McpToolError::client)?;
        if !requested.is_empty() {
            children.retain(|child| requested.iter().any(|id| id == &child.id));
        }
        let settled = children
            .iter()
            .all(|child| child_has_reached(mode, &child.status));
        if settled || std::time::Instant::now() >= deadline {
            let rows = children
                .into_iter()
                .map(|session| {
                    json!({
                        "id": session.id,
                        "title": session.title,
                        "status": session.status,
                        "workspace": session.workspace,
                        "parentSessionId": parent,
                        "relation": "child"
                    })
                })
                .collect::<Vec<_>>();
            return Ok(json!({
                "settled": settled,
                "timedOut": !settled,
                "children": rows,
                "waitedFor": mode
            }));
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn lineage_relation(
    context: &McpRuntimeContext,
    target_id: &str,
) -> Result<String, McpToolError> {
    let Some(caller) = &context.session_id else {
        return Ok("unrelated".to_string());
    };
    if caller == target_id {
        return Ok("self".to_string());
    }
    let client = runtime_client(context)?;
    let caller_parent = client
        .parent_session_id(caller)
        .await
        .map_err(McpToolError::client)?;
    let target_parent = client
        .parent_session_id(target_id)
        .await
        .map_err(McpToolError::client)?;
    if caller_parent.as_deref() == Some(target_id) {
        return Ok("parent".to_string());
    }
    let mut ancestor = caller_parent.clone();
    while let Some(current) = ancestor {
        if current == target_id {
            return Ok("ancestor".to_string());
        }
        ancestor = client
            .parent_session_id(&current)
            .await
            .map_err(McpToolError::client)?;
    }
    if target_parent.as_deref() == Some(caller.as_str()) {
        return Ok("child".to_string());
    }
    if caller_parent.is_some() && caller_parent == target_parent {
        return Ok("sibling".to_string());
    }
    Ok("unrelated".to_string())
}

fn frame_lineage_message(context: &McpRuntimeContext, text: &str, relation: &str) -> String {
    if matches!(relation, "parent" | "child") || context.session_id.is_none() {
        return text.to_string();
    }
    let caller = context.session_id.as_deref().unwrap_or("unknown");
    format!("[message from id:{caller}, channel: homie]\n\n{text}")
}

fn child_has_reached(mode: &str, status: &str) -> bool {
    match mode {
        "exited" => status == "exited",
        "done" => matches!(status, "idle" | "exited"),
        _ => matches!(
            status,
            "idle" | "needs_input" | "exited" | "archived" | "hibernated"
        ),
    }
}

async fn whoami_payload(context: &McpRuntimeContext) -> Result<Value, McpToolError> {
    let Some(session_id) = &context.session_id else {
        return Ok(json!({
            "sessionId": null,
            "parentSessionId": context.parent_session_id,
            "title": "unbound"
        }));
    };
    let Some(client) = &context.client else {
        return Ok(json!({
            "sessionId": session_id,
            "parentSessionId": context.parent_session_id,
            "title": "bound"
        }));
    };
    let session = client
        .list_sessions()
        .await
        .map_err(McpToolError::client)?
        .into_iter()
        .find(|session| session.id == *session_id);
    Ok(match session {
        Some(session) => json!({
            "sessionId": session.id,
            "parentSessionId": context.parent_session_id,
            "title": session.title,
            "status": session.status,
            "workspace": session.workspace,
        }),
        None => json!({
            "sessionId": session_id,
            "parentSessionId": context.parent_session_id,
            "title": "bound",
            "status": "unknown"
        }),
    })
}

fn runtime_client(context: &McpRuntimeContext) -> Result<&HomieClient, McpToolError> {
    context
        .client
        .as_ref()
        .ok_or_else(|| McpToolError::runtime("runtime unavailable: pass --data-dir"))
}

fn required_string(arguments: &Value, key: &str) -> Result<String, McpToolError> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| McpToolError::invalid_params(format!("{key} is required")))
}

fn required_string_any(arguments: &Value, keys: &[&str]) -> Result<String, McpToolError> {
    keys.iter()
        .find_map(|key| {
            arguments
                .get(*key)
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .map(str::to_string)
        })
        .ok_or_else(|| McpToolError::invalid_params(format!("{} is required", keys.join(" or "))))
}

fn session_status_text(status: &homie_proto::SessionStatus) -> Result<String, McpToolError> {
    serde_json::to_value(status)
        .ok()
        .and_then(|value| value.as_str().map(str::to_string))
        .ok_or_else(|| McpToolError::runtime("failed to encode session status"))
}

fn artifact_kind_text(kind: &ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::PullRequest => "pull_request",
        ArtifactKind::Preview => "preview",
        ArtifactKind::Link => "link",
    }
}

fn mcp_error_response(id: Value, error: McpToolError) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": error.code,
            "message": error.message
        }
    })
}

fn mcp_text_result(id: Value, payload: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [
                {
                    "type": "text",
                    "text": serde_json::to_string(&payload).unwrap_or_else(|_| "{}".to_string())
                }
            ]
        }
    })
}

struct McpToolRegistration {
    name: &'static str,
    description: &'static str,
    methods: &'static [&'static str],
}

const MCP_TOOLS: &[McpToolRegistration] = &[
    McpToolRegistration {
        name: "spawn_agent",
        description: "Spawn a Homie runtime-backed agent session.",
        methods: &[homie_proto::Method::SESSION_SPAWN],
    },
    McpToolRegistration {
        name: "list_agents",
        description: "List Homie sessions visible to this tool.",
        methods: &[homie_proto::Method::SESSION_LIST],
    },
    McpToolRegistration {
        name: "get_status",
        description: "Return a Homie session status report.",
        methods: &[homie_proto::Method::SESSION_STATUS],
    },
    McpToolRegistration {
        name: "send_prompt",
        description: "Send text to a Homie session.",
        methods: &[
            homie_proto::Method::SESSION_SEND_TEXT,
            homie_proto::Method::SESSION_PARENT,
        ],
    },
    McpToolRegistration {
        name: "wait_for_agent",
        description: "Wait for a Homie session to complete.",
        methods: &[homie_proto::Method::SESSION_STATUS],
    },
    McpToolRegistration {
        name: "read_output",
        description: "Read output from a Homie session.",
        methods: &[homie_proto::Method::SESSION_SNAPSHOT],
    },
    McpToolRegistration {
        name: "create_worktree",
        description: "Create a git worktree for a session.",
        methods: &[homie_proto::Method::WORKTREE_CREATE],
    },
    McpToolRegistration {
        name: "list_worktrees",
        description: "List Homie worktrees.",
        methods: &[homie_proto::Method::WORKTREE_LIST],
    },
    McpToolRegistration {
        name: "remove_worktree",
        description: "Remove a Homie worktree.",
        methods: &[homie_proto::Method::WORKTREE_REMOVE],
    },
    McpToolRegistration {
        name: "get_artifacts",
        description: "Return artifacts and ports found in session output.",
        methods: &[homie_proto::Method::SESSION_ARTIFACTS],
    },
    McpToolRegistration {
        name: "release_agent",
        description: "Release or close an agent session.",
        methods: &[
            homie_proto::Method::SESSION_KILL,
            homie_proto::Method::SESSION_PARENT,
        ],
    },
    McpToolRegistration {
        name: "whoami",
        description: "Return the current Homie MCP identity.",
        methods: &[],
    },
    McpToolRegistration {
        name: "list_children",
        description: "List child sessions for the current MCP identity.",
        methods: &[homie_proto::Method::SESSION_LIST_CHILDREN],
    },
    McpToolRegistration {
        name: "wait_for_children",
        description: "Wait for child sessions to finish.",
        methods: &[homie_proto::Method::SESSION_LIST_CHILDREN],
    },
];

fn mcp_tool_is_available(context: &McpRuntimeContext, name: &str) -> bool {
    let hello = context.client.as_ref().and_then(HomieClient::hello);
    mcp_tool_is_available_for_capabilities(
        name,
        context.client.is_some(),
        hello
            .as_ref()
            .map(|hello| hello.method_capabilities.as_slice()),
        context.session_id.is_some(),
    )
}

fn mcp_tool_is_available_for_capabilities(
    name: &str,
    has_client: bool,
    method_capabilities: Option<&[String]>,
    has_bound_session: bool,
) -> bool {
    let Some(tool) = MCP_TOOLS.iter().find(|tool| tool.name == name) else {
        return false;
    };
    if !has_client {
        return tool.methods.is_empty();
    }
    let bound_methods: &[&str] = match (name, has_bound_session) {
        ("whoami", true) => &[homie_proto::Method::SESSION_LIST],
        _ => &[],
    };
    if tool.methods.is_empty() && bound_methods.is_empty() {
        return true;
    }
    let Some(method_capabilities) = method_capabilities else {
        return false;
    };
    tool.methods.iter().chain(bound_methods).all(|method| {
        method_capabilities
            .iter()
            .any(|capability| capability == method)
    })
}

fn mcp_tool_descriptors(context: &McpRuntimeContext) -> Vec<Value> {
    MCP_TOOLS
        .iter()
        .filter(|tool| mcp_tool_is_available(context, tool.name))
        .map(|tool| {
            json!({
                "name": tool.name,
                "description": tool.description,
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": true
                }
            })
        })
        .collect()
}

fn mcp_tool_names(context: &McpRuntimeContext) -> Vec<&'static str> {
    let hello = context.client.as_ref().and_then(HomieClient::hello);
    mcp_tool_names_for_capabilities(
        context.client.is_some(),
        hello
            .as_ref()
            .map(|hello| hello.method_capabilities.as_slice()),
        context.session_id.is_some(),
    )
}

fn mcp_tool_names_for_capabilities(
    has_client: bool,
    method_capabilities: Option<&[String]>,
    has_bound_session: bool,
) -> Vec<&'static str> {
    MCP_TOOLS
        .iter()
        .filter(|tool| {
            mcp_tool_is_available_for_capabilities(
                tool.name,
                has_client,
                method_capabilities,
                has_bound_session,
            )
        })
        .map(|tool| tool.name)
        .collect()
}

fn app_launch() -> anyhow::Result<()> {
    let data_dir = default_data_dir()?;
    let storage = open_or_create(StorageConfig {
        data_dir: data_dir.clone(),
    })
    .with_context(|| format!("open storage at {}", data_dir.display()))?;
    storage.migrate().context("migrate storage")?;
    storage.seed_defaults().context("seed default config")?;
    let health = storage.health_check().context("check storage health")?;

    let message = format!(
        "Homie local V1 is ready.\n\nStorage initialized at:\n{}\n\nSchema: {}\nForeign keys: {}\nJournal: {}",
        data_dir.display(),
        health.schema_version,
        health.foreign_keys,
        health.journal_mode
    );

    if cfg!(target_os = "macos") {
        let script = format!(
            "display dialog {} buttons {{\"OK\"}} default button \"OK\" with title \"Homie\" with icon note",
            applescript_string(&message)
        );
        let _ = std::process::Command::new("/usr/bin/osascript")
            .arg("-e")
            .arg(script)
            .status();
    } else {
        println!("{message}");
    }

    Ok(())
}

fn applescript_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

fn default_data_dir() -> anyhow::Result<PathBuf> {
    let home = account_home_directory()?;
    #[cfg(target_os = "macos")]
    let data_dir = home
        .join("Library")
        .join("Application Support")
        .join("Homie");
    #[cfg(not(target_os = "macos"))]
    let data_dir = home.join(".local").join("share").join("homie");
    if !data_dir.is_absolute() {
        anyhow::bail!("OS account home directory is not absolute");
    }
    Ok(data_dir)
}

#[cfg(unix)]
fn account_home_directory() -> anyhow::Result<PathBuf> {
    use std::ffi::{CStr, OsStr};
    use std::os::unix::ffi::OsStrExt as _;

    // SAFETY: geteuid has no preconditions and does not access caller memory.
    let uid = unsafe { libc::geteuid() };
    // SAFETY: sysconf reads a process-global constant and has no pointer arguments.
    let configured_size = unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) };
    let mut capacity = usize::try_from(configured_size)
        .ok()
        .filter(|size| *size > 0)
        .unwrap_or(16 * 1024)
        .clamp(16 * 1024, 1024 * 1024);
    loop {
        let mut record = std::mem::MaybeUninit::<libc::passwd>::zeroed();
        let mut result = std::ptr::null_mut();
        let mut buffer = vec![0_u8; capacity];
        // SAFETY: record and buffer are valid writable storage for this call; result is checked
        // before the initialized record and pw_dir pointer are read.
        let code = unsafe {
            libc::getpwuid_r(
                uid,
                record.as_mut_ptr(),
                buffer.as_mut_ptr().cast(),
                buffer.len(),
                &mut result,
            )
        };
        if code == libc::ERANGE && capacity < 1024 * 1024 {
            capacity *= 2;
            continue;
        }
        if code != 0 {
            return Err(std::io::Error::from_raw_os_error(code))
                .context("resolve OS account home directory");
        }
        if result.is_null() {
            anyhow::bail!("OS account has no passwd entry");
        }
        // SAFETY: getpwuid_r succeeded and returned result pointing at initialized record storage.
        let record = unsafe { record.assume_init() };
        if record.pw_dir.is_null() {
            anyhow::bail!("OS account passwd entry has no home directory");
        }
        // SAFETY: pw_dir points into buffer and is NUL-terminated for the buffer lifetime.
        let bytes = unsafe { CStr::from_ptr(record.pw_dir) }.to_bytes();
        let home = PathBuf::from(OsStr::from_bytes(bytes));
        if !home.is_absolute() {
            anyhow::bail!("OS account home directory is not absolute");
        }
        return Ok(home);
    }
}

#[cfg(not(unix))]
fn account_home_directory() -> anyhow::Result<PathBuf> {
    anyhow::bail!("default Homie data directory is unsupported on this platform")
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn default_data_dir_uses_absolute_os_account_home() {
        let account_home = account_home_directory().expect("OS account home");
        let data_dir = default_data_dir().expect("default data dir");

        assert!(data_dir.is_absolute());
        assert!(data_dir.starts_with(account_home));
    }

    #[test]
    fn cli_parses_automation_commands() {
        assert!(matches!(
            Cli::try_parse_from(["homie", "hook", "SessionStart"])
                .expect("hook")
                .command,
            Some(Command::Hook(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["homie", "notify", "{\"type\":\"done\"}"])
                .expect("notify")
                .command,
            Some(Command::Notify(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["homie", "mcp-tools"])
                .expect("mcp-tools")
                .command,
            Some(Command::McpTools)
        ));
        assert!(matches!(
            Cli::try_parse_from(["homie", "mcp-call", "--tool", "list_agents"])
                .expect("mcp-call")
                .command,
            Some(Command::McpCall(_))
        ));
        assert!(matches!(
            Cli::try_parse_from([
                "homie",
                "session",
                "snapshot",
                "--id",
                "session-1",
                "--offset",
                "4",
                "--max-bytes",
                "64",
            ])
            .expect("snapshot")
            .command,
            Some(Command::Session(SessionCommand {
                command: SessionSubcommand::Snapshot(_)
            }))
        ));
        assert!(matches!(
            Cli::try_parse_from(["homie", "session", "kill", "--id", "session-1"])
                .expect("kill")
                .command,
            Some(Command::Session(SessionCommand {
                command: SessionSubcommand::Kill(_)
            }))
        ));
        assert!(matches!(
            Cli::try_parse_from(["homie", "session", "diff", "--id", "session-1"])
                .expect("diff")
                .command,
            Some(Command::Session(SessionCommand {
                command: SessionSubcommand::Diff(_)
            }))
        ));
        assert!(matches!(
            Cli::try_parse_from(["homie", "events", "list", "--after-seq", "7"])
                .expect("events list")
                .command,
            Some(Command::Events(_))
        ));
        assert!(matches!(
            Cli::try_parse_from(["homie", "control-stdio"])
                .expect("control stdio")
                .command,
            Some(Command::ControlStdio(_))
        ));
        assert!(matches!(
            Cli::try_parse_from([
                "homie",
                "host",
                "locate-repo",
                "--origin-url",
                "git@example.invalid:acme/app.git"
            ])
            .expect("host locate repo")
            .command,
            Some(Command::Host(HostCommand {
                command: HostSubcommand::LocateRepo(_)
            }))
        ));
        assert!(matches!(
            Cli::try_parse_from(["homie", "worktree", "list", "--repo", "."])
                .expect("worktree list")
                .command,
            Some(Command::Worktree(WorktreeCommand {
                command: WorktreeSubcommand::List(_)
            }))
        ));
        assert!(matches!(
            Cli::try_parse_from(["homie", "ports"])
                .expect("ports")
                .command,
            Some(Command::Ports(_))
        ));
        assert!(matches!(
            Cli::try_parse_from([
                "homie",
                "agent",
                "readiness",
                "--descriptor-dir",
                "assets/agent-descriptors"
            ])
            .expect("agent readiness")
            .command,
            Some(Command::Agent(AgentCommand {
                command: AgentSubcommand::Readiness(_)
            }))
        ));
        assert!(matches!(
            Cli::try_parse_from(["homie", "usage", "summary"])
                .expect("usage summary")
                .command,
            Some(Command::Usage(UsageCommand {
                command: UsageSubcommand::Summary(_)
            }))
        ));
    }

    #[tokio::test]
    async fn mcp_stdio_response_omits_tools_without_required_runtime_methods() {
        let response = mcp_stdio_response(
            r#"{"jsonrpc":"2.0","id":"a","method":"tools/call","params":{"name":"list_agents","arguments":{}}}"#,
        )
        .await;
        assert_eq!(response["error"]["code"], -32601);
    }

    #[test]
    fn mcp_capability_bound_whoami_requires_session_list() {
        let capabilities = vec![homie_proto::Method::SESSION_SPAWN.to_string()];

        assert!(!mcp_tool_is_available_for_capabilities(
            "whoami",
            true,
            Some(&capabilities),
            true,
        ));
    }

    #[test]
    fn mcp_capability_bound_whoami_accepts_session_list() {
        let capabilities = vec![homie_proto::Method::SESSION_LIST.to_string()];

        assert!(mcp_tool_is_available_for_capabilities(
            "whoami",
            true,
            Some(&capabilities),
            true,
        ));
    }

    #[test]
    fn mcp_capability_unbound_whoami_needs_no_runtime_method() {
        let capabilities = Vec::new();

        assert!(mcp_tool_is_available_for_capabilities(
            "whoami",
            true,
            Some(&capabilities),
            false,
        ));
    }

    #[test]
    fn mcp_capability_whoami_without_client_needs_no_runtime_method() {
        assert!(mcp_tool_is_available_for_capabilities(
            "whoami", false, None, true,
        ));
    }

    #[test]
    fn mcp_capability_spawn_without_parent_only_requires_spawn() {
        let capabilities = vec![homie_proto::Method::SESSION_SPAWN.to_string()];

        assert!(mcp_tool_is_available_for_capabilities(
            "spawn_agent",
            true,
            Some(&capabilities),
            false,
        ));
    }

    #[test]
    fn mcp_capability_spawn_with_parent_is_one_atomic_spawn_method() {
        let capabilities = vec![homie_proto::Method::SESSION_SPAWN.to_string()];

        assert!(mcp_tool_is_available_for_capabilities(
            "spawn_agent",
            true,
            Some(&capabilities),
            true,
        ));
    }

    #[test]
    fn mcp_capability_tools_list_and_call_use_same_partial_hello_decision() {
        let capabilities = vec![homie_proto::Method::SESSION_SPAWN.to_string()];
        let listed = mcp_tool_names_for_capabilities(true, Some(&capabilities), false);

        for name in ["spawn_agent", "whoami", "list_agents"] {
            assert_eq!(
                listed.contains(&name),
                mcp_tool_is_available_for_capabilities(name, true, Some(&capabilities), false,),
                "tools/list and tools/call differ for {name}",
            );
        }
    }

    #[test]
    fn hook_command_outputs_redacted_structured_event() {
        let output = hook_output(
            "PermissionRequest",
            Some(
                r#"{"session_id":"s1","tool_name":"Bash","tool_input":{"command":"deploy --token=sk-secret"}}"#,
            ),
        )
        .expect("hook output");
        assert_eq!(output["event"]["kind"], "permissionRequest");
        assert_eq!(output["sessionId"], "s1");
        assert_eq!(output["needsInput"]["kind"], "approval");
        let serialized = serde_json::to_string(&output).expect("serialize");
        assert!(!serialized.contains("sk-secret"));
        assert!(serialized.contains("deploy --token"));
    }

    #[test]
    fn notify_command_outputs_codex_turn_complete() {
        let output = notify_output(Some(
            r#"{"type":"agent-turn-complete","thread-id":"thread-1","input-messages":["Implement CLI hook"]}"#,
        ))
        .expect("notify output");
        assert_eq!(output["event"]["kind"], "codexTurnComplete");
        assert_eq!(output["sessionId"], "thread-1");
        assert_eq!(output["firstPromptTitle"], "Implement CLI hook");
    }

    #[test]
    fn hook_command_fails_open_for_unknown_event() {
        let output = hook_output(
            "FutureHook",
            Some(r#"{"session_id":"s2","authorization":"Bearer example-token"}"#),
        )
        .expect("hook output");
        assert_eq!(output["event"]["kind"], "unknown");
        assert_eq!(output["sessionId"], "s2");
        let serialized = serde_json::to_string(&output).expect("serialize");
        assert!(!serialized.contains("example-token"));
    }
}
