use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

mod commands;
mod server;
mod spotify;

use server::{SpotifyServer, process_command};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    info!("Starting BlockySpot...");

    // Create server instance
    let server = Arc::new(SpotifyServer::new());

    // Start TCP server
    let listener = TcpListener::bind("127.0.0.1:8888").await?;
    info!("Listening on port 8888");

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                info!("New client connected: {addr}");
                let server = server.clone();
                tokio::spawn(async move {
                    if let Err(e) = handle_connection(socket, server).await {
                        error!("Error handling connection: {e}");
                    }
                });
            }
            Err(e) => {
                error!("Failed to accept connection: {e}");
            }
        }
    }
}

async fn handle_connection(
    mut socket: tokio::net::TcpStream,
    server: Arc<SpotifyServer>,
) -> Result<()> {
    let mut buffer = [0; 1024];

    loop {
        let n = socket.read(&mut buffer).await?;
        if n == 0 {
            return Ok(());
        }

        let command_str = String::from_utf8_lossy(&buffer[..n]);
        let response = process_command(&command_str, &server).await;
        let response_json = serde_json::to_string(&response)?;

        socket.write_all(response_json.as_bytes()).await?;
        socket.write_all(b"\n").await?;
    }
}
