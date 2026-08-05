use anyhow::Context;
use clap::{Parser, Subcommand};
use homie_storage::{CreateSession, SessionSummary, StorageConfig, open_or_create};
use serde::Serialize;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "homie")]
#[command(about = "Homie local development and diagnostics CLI")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Doctor(DoctorArgs),
    Runtime(RuntimeCommand),
    Session(SessionCommand),
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
struct CommonArgs {
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long)]
    json: bool,
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
    database_ready: bool,
    schema_version: i64,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Doctor(args) => doctor(args),
        Command::Runtime(command) => match command.command {
            RuntimeSubcommand::Status(args) => runtime_status(args),
        },
        Command::Session(command) => match command.command {
            SessionSubcommand::Create(args) => session_create(args),
            SessionSubcommand::List(args) => session_list(args),
        },
    }
}

fn doctor(args: DoctorArgs) -> anyhow::Result<()> {
    let data_dir = args.data_dir.unwrap_or_else(default_data_dir);
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

fn runtime_status(args: CommonArgs) -> anyhow::Result<()> {
    let storage = open_ready_storage(args.data_dir)?;
    let health = storage.health_check().context("check storage health")?;
    let output = RuntimeStatusOutput {
        status: "ok".to_string(),
        runtime_process: "not_running".to_string(),
        database_ready: true,
        schema_version: health.schema_version,
    };
    if args.json {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("runtime: {}", output.runtime_process);
        println!("database: ready");
        println!("schema version: {}", output.schema_version);
    }
    Ok(())
}

fn session_create(args: SessionCreateArgs) -> anyhow::Result<()> {
    let storage = open_ready_storage(args.data_dir)?;
    let session = storage
        .create_session(CreateSession {
            workspace: args.workspace,
            title: args.title,
        })
        .context("create session")?;
    print_session_or_json(&session, args.json)
}

fn session_list(args: CommonArgs) -> anyhow::Result<()> {
    let storage = open_ready_storage(args.data_dir)?;
    let sessions = storage.list_sessions().context("list sessions")?;
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

fn open_ready_storage(data_dir: Option<PathBuf>) -> anyhow::Result<homie_storage::Storage> {
    let data_dir = data_dir.unwrap_or_else(default_data_dir);
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

fn default_data_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home)
            .join("Library")
            .join("Application Support")
            .join("Homie");
    }
    PathBuf::from(".homie")
}
