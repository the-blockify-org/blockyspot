use anyhow::Result;
use log::info;

mod command_manager;
mod commands;
mod server;
mod spotify;
mod ws_sink;

use server::SpotifyServer;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting BlockySpot...");

    let server = SpotifyServer::new();
    info!("Starting WebSocket server on port 8888...");
    server.start(8888).await;

    Ok(())
}
