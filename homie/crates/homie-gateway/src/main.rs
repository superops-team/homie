//! Homie local LLM gateway binary: an OpenAI/Anthropic-compatible proxy that
//! issues virtual keys and forwards to a configured OpenAI-compatible upstream.
//!
//! `homie-gateway inject --agent <codex|claude>` prints the injection preview
//! (the exact args/env real spawn-time injection would apply) and exits.

use std::env;

use tokio::net::TcpListener;

use homie_gateway::config::{
    CredentialSource, GatewayConfig, config_path, db_path, listen_or_default,
};
use homie_gateway::db::Db;
use homie_gateway::inject;
use homie_gateway::state::AppState;
use homie_gateway::upstream::Upstream;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("homie-gateway: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("inject") {
        print!("{}", inject_preview(&args)?);
        return Ok(());
    }
    serve().await
}

/// Handle the `inject --agent <codex|claude>` subcommand, printing one JSON
/// line to stdout. Preview must not require upstream credentials, so only the
/// listen address is read (defaulting when absent).
fn inject_preview(args: &[String]) -> Result<String, Box<dyn std::error::Error>> {
    let agent = flag_value(args, "--agent").ok_or("inject requires --agent <codex|claude>")?;
    let agent = inject::Agent::parse(agent).ok_or_else(|| format!("unknown agent: {agent}"))?;
    let base_url = format!("http://{}", listen_or_default());
    let value = inject::preview(&agent, &base_url);
    Ok(serde_json::to_string(&value)?)
}

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|w| w[0] == flag)
        .map(|w| w[1].as_str())
}

async fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let config = GatewayConfig::load(&config_path())?;
    let db = Db::open(&db_path())?;
    let prefer_node = config.credential_source == CredentialSource::Node;
    let upstream = Upstream::new(config.base_url.clone(), config.api_key.clone(), prefer_node);
    let state = AppState::new(
        db,
        upstream,
        config.master_key.clone(),
        config.models.clone(),
        config.policy.clone(),
    );

    let listener = TcpListener::bind(config.listen).await?;
    eprintln!("homie-gateway listening on {}", config.listen);
    let app = homie_gateway::routes::router(state);
    axum::serve(listener, app).await?;
    Ok(())
}
