use anyhow::Result;
use clap::Parser;
use codex_bridge_cli::{bridge, config::BridgeConfig};
use std::path::PathBuf;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

/// Default log level used when RUST_LOG is not set.
const DEFAULT_LOG_LEVEL: &str = "info";

#[derive(Debug, Parser)]
#[command(name = "codex-bridge")]
#[command(about = "Codex Responses API to Chat Completions protocol bridge")]
struct Args {
    #[arg(short, long)]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let args = Args::parse();
    let config = BridgeConfig::from_path(&args.config)?;
    let state = bridge::BridgeState::new(config)?;
    let addr = state.bind_addr();

    info!(
        address = %addr,
        config = %args.config.display(),
        "codex-bridge starting"
    );

    bridge::serve_with_graceful_shutdown(state).await?;

    info!("codex-bridge stopped");
    Ok(())
}

fn init_tracing() {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(DEFAULT_LOG_LEVEL));
    fmt::Subscriber::builder()
        .with_env_filter(filter)
        .with_target(false)
        .init();
}
