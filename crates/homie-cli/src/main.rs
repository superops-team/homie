use anyhow::Context;
use clap::{Parser, Subcommand};
use homie_storage::{StorageConfig, open_or_create};
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
}

#[derive(Parser, Debug)]
struct DoctorArgs {
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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Doctor(args) => doctor(args),
    }
}

fn doctor(args: DoctorArgs) -> anyhow::Result<()> {
    let data_dir = args.data_dir.unwrap_or_else(default_data_dir);
    let storage = open_or_create(StorageConfig {
        data_dir: data_dir.clone(),
    })
    .with_context(|| format!("open storage at {}", data_dir.display()))?;
    storage.migrate().context("migrate storage")?;
    let health = storage.health_check().context("check storage health")?;
    let output = DoctorOutput {
        status: "ok".to_string(),
        database_path: health.database_path.display().to_string(),
        schema_version: health.schema_version,
        foreign_keys: health.foreign_keys,
        journal_mode: health.journal_mode,
    };

    if args.json {
        println!("{}", serde_json::to_string_pretty(&output)?);
    } else {
        println!("Homie doctor: {}", output.status);
        println!("database: {}", output.database_path);
        println!("schema version: {}", output.schema_version);
        println!("foreign keys: {}", output.foreign_keys);
        println!("journal mode: {}", output.journal_mode);
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
