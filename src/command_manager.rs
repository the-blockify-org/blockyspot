use crate::commands::{Command, CommandResponse};
use crate::spotify::SpotifyClient;

pub trait CommandHandler {
    fn handle(client: &SpotifyClient, command: &Command) -> CommandResponse;
}

macro_rules! impl_simple_handler {
    ($name:ident, $method:ident, $success_msg:expr) => {
        pub struct $name;
        impl CommandHandler for $name {
            fn handle(client: &SpotifyClient, _command: &Command) -> CommandResponse {
                match client.$method() {
                    Ok(()) => CommandResponse::success($success_msg, None),
                    Err(e) => {
                        CommandResponse::error(format!("Failed to {}: {}", stringify!($method), e))
                    }
                }
            }
        }
    };
}

impl_simple_handler!(PlayCommandHandler, play, "Playback started");
impl_simple_handler!(PlayPauseCommandHandler, play_pause, "Playback toggled");
impl_simple_handler!(PauseCommandHandler, pause, "Playback paused");
impl_simple_handler!(PrevCommandHandler, prev, "Previous track");
impl_simple_handler!(NextCommandHandler, next, "Next track");
impl_simple_handler!(VolumeUpCommandHandler, volume_up, "Volume increased");
impl_simple_handler!(VolumeDownCommandHandler, volume_down, "Volume decreased");
impl_simple_handler!(ShutdownCommandHandler, shutdown, "Device shutdown");
impl_simple_handler!(ActivateCommandHandler, activate, "Device activated");

// Handlers for commands with parameters
pub struct ShuffleCommandHandler;
impl CommandHandler for ShuffleCommandHandler {
    fn handle(client: &SpotifyClient, command: &Command) -> CommandResponse {
        if let Command::Shuffle(state) = command {
            match client.shuffle(*state) {
                Ok(()) => CommandResponse::success(
                    format!("Shuffle {}", if *state { "enabled" } else { "disabled" }),
                    None,
                ),
                Err(e) => CommandResponse::error(format!("Failed to set shuffle: {}", e)),
            }
        } else {
            CommandResponse::error("Invalid shuffle command")
        }
    }
}

pub struct RepeatCommandHandler;
impl CommandHandler for RepeatCommandHandler {
    fn handle(client: &SpotifyClient, command: &Command) -> CommandResponse {
        if let Command::Repeat(state) = command {
            match client.repeat(*state) {
                Ok(()) => CommandResponse::success(
                    format!("Repeat {}", if *state { "enabled" } else { "disabled" }),
                    None,
                ),
                Err(e) => CommandResponse::error(format!("Failed to set repeat: {}", e)),
            }
        } else {
            CommandResponse::error("Invalid repeat command")
        }
    }
}

pub struct RepeatTrackCommandHandler;
impl CommandHandler for RepeatTrackCommandHandler {
    fn handle(client: &SpotifyClient, command: &Command) -> CommandResponse {
        if let Command::RepeatTrack(state) = command {
            match client.repeat_track(*state) {
                Ok(()) => CommandResponse::success(
                    format!(
                        "Track repeat {}",
                        if *state { "enabled" } else { "disabled" }
                    ),
                    None,
                ),
                Err(e) => CommandResponse::error(format!("Failed to set track repeat: {}", e)),
            }
        } else {
            CommandResponse::error("Invalid repeat track command")
        }
    }
}

pub struct DisconnectCommandHandler;
impl CommandHandler for DisconnectCommandHandler {
    fn handle(client: &SpotifyClient, command: &Command) -> CommandResponse {
        if let Command::Disconnect { pause } = command {
            match client.disconnect(*pause) {
                Ok(()) => CommandResponse::success("Device disconnected", None),
                Err(e) => CommandResponse::error(format!("Failed to disconnect: {}", e)),
            }
        } else {
            CommandResponse::error("Invalid disconnect command")
        }
    }
}

pub struct SetPositionCommandHandler;
impl CommandHandler for SetPositionCommandHandler {
    fn handle(client: &SpotifyClient, command: &Command) -> CommandResponse {
        if let Command::SetPosition(position) = command {
            match client.set_position_ms(*position) {
                Ok(()) => CommandResponse::success("Position updated", None),
                Err(e) => CommandResponse::error(format!("Failed to set position: {}", e)),
            }
        } else {
            CommandResponse::error("Invalid set position command")
        }
    }
}

pub struct SetVolumeCommandHandler;
impl CommandHandler for SetVolumeCommandHandler {
    fn handle(client: &SpotifyClient, command: &Command) -> CommandResponse {
        if let Command::SetVolume(volume) = command {
            match client.set_volume(*volume) {
                Ok(()) => CommandResponse::success("Volume updated", None),
                Err(e) => CommandResponse::error(format!("Failed to set volume: {}", e)),
            }
        } else {
            CommandResponse::error("Invalid set volume command")
        }
    }
}

#[derive(Clone)]
pub struct CommandManager;
impl CommandManager {
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, command: Command, client: &SpotifyClient) -> CommandResponse {
        match command {
            Command::Play => PlayCommandHandler::handle(client, &command),
            Command::PlayPause => PlayPauseCommandHandler::handle(client, &command),
            Command::Pause => PauseCommandHandler::handle(client, &command),
            Command::Prev => PrevCommandHandler::handle(client, &command),
            Command::Next => NextCommandHandler::handle(client, &command),
            Command::VolumeUp => VolumeUpCommandHandler::handle(client, &command),
            Command::VolumeDown => VolumeDownCommandHandler::handle(client, &command),
            Command::Shutdown => ShutdownCommandHandler::handle(client, &command),
            Command::Shuffle(_) => ShuffleCommandHandler::handle(client, &command),
            Command::Repeat(_) => RepeatCommandHandler::handle(client, &command),
            Command::RepeatTrack(_) => RepeatTrackCommandHandler::handle(client, &command),
            Command::Disconnect { .. } => DisconnectCommandHandler::handle(client, &command),
            Command::SetPosition(_) => SetPositionCommandHandler::handle(client, &command),
            Command::SetVolume(_) => SetVolumeCommandHandler::handle(client, &command),
            Command::Activate => ActivateCommandHandler::handle(client, &command),
            Command::CreateDevice { .. } => {
                CommandResponse::error("CreateDevice command should be handled by the server")
            }
        }
    }
}
