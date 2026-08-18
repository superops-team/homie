//! Homie local LLM gateway binary: an OpenAI/Anthropic-compatible proxy that
//! issues virtual keys and forwards to a configured OpenAI-compatible upstream.

use tokio::net::TcpListener;

use homie_gateway::config::{GatewayConfig, config_path, db_path};
use homie_gateway::db::Db;
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
    let config = GatewayConfig::load(&config_path())?;
    let db = Db::open(&db_path())?;
    let upstream = Upstream::new(config.base_url.clone(), config.api_key.clone());
    let state = AppState::new(db, upstream, config.master_key.clone());

    let listener = TcpListener::bind(config.listen).await?;
    eprintln!("homie-gateway listening on {}", config.listen);
    let app = homie_gateway::routes::router(state);
    axum::serve(listener, app).await?;
    Ok(())
}
