use anyhow::Result;
use clap::Parser;
use codex_bridge_cli::{bridge, config::BridgeConfig};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "codex-bridge")]
#[command(about = "Codex Responses API to Chat Completions protocol bridge")]
struct Args {
    #[arg(short, long)]
    config: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = BridgeConfig::from_path(args.config)?;
    let state = bridge::BridgeState::new(config)?;
    eprintln!("codex-bridge listening on http://{}", state.bind_addr());
    bridge::serve(state).await?;
    Ok(())
}
