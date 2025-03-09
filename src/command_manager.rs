use crate::spotify::SpotifyClient;
use crate::commands::{Command, CommandResponse};

pub trait CommandHandler {
    fn handle(client: &SpotifyClient, args: CommandArgs) -> CommandResponse;
}

#[derive(Debug)]
pub struct CommandArgs {
    pub data: serde_json::Value,
}

pub struct PauseCommandHandler;
impl CommandHandler for PauseCommandHandler {
    fn handle(client: &SpotifyClient, _args: CommandArgs) -> CommandResponse {
        match client.pause() {
            Ok(()) => CommandResponse::success("Playback paused", None),
            Err(e) => CommandResponse::error(format!("Failed to pause: {}", e)),
        }
    }
}

pub struct CommandManager;
impl CommandManager {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, command: Command, client: &SpotifyClient) -> CommandResponse {
        let args = CommandArgs {
            data: serde_json::to_value(&command).unwrap_or_default(),
        };

        match command {
            Command::Pause => PauseCommandHandler::handle(client, args),
            _ => CommandResponse::error("Command not implemented yet"),
        }
    }
} 