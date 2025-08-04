use anyhow::Result;
use clap::Parser;
use log::info;

mod command_manager;
mod commands;
mod server;
mod spotify;
mod ws_sink;

use server::SpotifyServer;

#[derive(Parser)]
#[command(name = "blockyspot")]
#[command(about = "A modifiable Spotify Connect client using librespot")]
#[command(version = "0.1.0")]
struct Args {
    /// Port to run the WebSocket server on
    #[arg(short, long, default_value_t = 8888)]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    info!("Starting BlockySpot...");

    let server = SpotifyServer::new();
    info!("Starting WebSocket server on port {}...", args.port);
    server.start(args.port).await;

    Ok(())
}
