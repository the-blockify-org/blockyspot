use crate::commands::{Command, CommandResponse};
use crate::spotify::SpotifyClient;
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct SpotifyServer {
    clients: Arc<Mutex<HashMap<String, SpotifyClient>>>,
}

impl SpotifyServer {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn handle_command(&self, command: Command) -> CommandResponse {
        let mut clients = self.clients.lock().await;

        match command {
            Command::Connect { token, device_id, device_name } => {
                if clients.contains_key(&device_id) {
                    return CommandResponse::error("Device ID already exists");
                }

                let mut client = SpotifyClient::new();
                match client.initialize(token, device_name.unwrap_or_else(|| format!("Blockyspot {}", device_id))).await {
                    Ok(_) => {
                        clients.insert(device_id.clone(), client);
                        CommandResponse::success("Connected to Spotify", None)
                    }
                    Err(e) => CommandResponse::error(&format!("Failed to connect: {}", e)),
                }
            }
            Command::Disconnect { device_id } => {
                if let Some(client) = clients.get_mut(&device_id) {
                    client.stop_playback();
                    clients.remove(&device_id);
                    CommandResponse::success("Disconnected from Spotify", None)
                } else {
                    CommandResponse::error("Device not found")
                }
            }
            Command::Play { device_id, track_id } => {
                if let Some(client) = clients.get_mut(&device_id) {
                    match client.play_track(track_id) {
                        Ok(_) => CommandResponse::success("Playing track", None),
                        Err(e) => CommandResponse::error(&format!("Failed to play track: {}", e)),
                    }
                } else {
                    CommandResponse::error("Device not found")
                }
            }
            Command::Pause { device_id } => {
                if let Some(client) = clients.get_mut(&device_id) {
                    match client.pause() {
                        Ok(_) => CommandResponse::success("Playback paused", None),
                        Err(e) => CommandResponse::error(&format!("Failed to pause: {}", e)),
                    }
                } else {
                    CommandResponse::error("Device not found")
                }
            }
            Command::Resume { device_id } => {
                if let Some(client) = clients.get_mut(&device_id) {
                    match client.resume() {
                        Ok(_) => CommandResponse::success("Playback resumed", None),
                        Err(e) => CommandResponse::error(&format!("Failed to resume: {}", e)),
                    }
                } else {
                    CommandResponse::error("Device not found")
                }
            }
            Command::Stop { device_id } => {
                if let Some(client) = clients.get_mut(&device_id) {
                    match client.stop_playback() {
                        Ok(_) => CommandResponse::success("Playback stopped", None),
                        Err(e) => CommandResponse::error(&format!("Failed to stop: {}", e)),
                    }
                } else {
                    CommandResponse::error("Device not found")
                }
            }
            Command::GetCurrentTrack { device_id } => {
                if let Some(_client) = clients.get(&device_id) {
                    // TODO: Implement getting current track info
                    CommandResponse::error("Getting current track not implemented yet")
                } else {
                    CommandResponse::error("Device not found")
                }
            }
        }
    }
}

pub async fn process_command(command_str: &str, server: &SpotifyServer) -> CommandResponse {
    match serde_json::from_str::<Command>(command_str) {
        Ok(command) => {
            info!("Processing command: {:?}", command);
            server.handle_command(command).await
        }
        Err(e) => {
            error!("Failed to parse command: {}", e);
            CommandResponse::error(&format!("Invalid command format: {}", e))
        }
    }
}
