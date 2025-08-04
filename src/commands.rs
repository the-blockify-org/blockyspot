use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct CommandMessage {
    #[serde(default)]
    pub device_id: Option<String>,
    pub command_type: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum Command {
    CreateDevice {
        token: String,
        device_name: Option<String>,
    },
    Play,
    PlayPause,
    Pause,
    Prev,
    Next,
    VolumeUp,
    VolumeDown,
    Shutdown,
    Shuffle(bool),
    Repeat(bool),
    RepeatTrack(bool),
    Disconnect {
        pause: bool,
    },
    SetPosition(u32),
    SetVolume(u16),
    Activate,
}

impl Command {
    pub fn from_message(msg: CommandMessage) -> Result<(String, Command), String> {
        let command = match msg.command_type.as_str() {
            "CreateDevice" => {
                let token = msg
                    .params
                    .get("token")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing token parameter")?
                    .to_string();

                let device_name = msg
                    .params
                    .get("device_name")
                    .and_then(|v| v.as_str())
                    .map(String::from);

                (String::new(), Command::CreateDevice { token, device_name })
            }
            cmd_type => {
                let command = match cmd_type {
                    "Play" => Command::Play,
                    "PlayPause" => Command::PlayPause,
                    "Pause" => Command::Pause,
                    "Prev" => Command::Prev,
                    "Next" => Command::Next,
                    "VolumeUp" => Command::VolumeUp,
                    "VolumeDown" => Command::VolumeDown,
                    "Shutdown" => Command::Shutdown,
                    "Shuffle" => {
                        let state = msg
                            .params
                            .get("state")
                            .and_then(|v| v.as_bool())
                            .ok_or("Missing or invalid shuffle state parameter")?;
                        Command::Shuffle(state)
                    }
                    "Repeat" => {
                        let state = msg
                            .params
                            .get("state")
                            .and_then(|v| v.as_bool())
                            .ok_or("Missing or invalid repeat state parameter")?;
                        Command::Repeat(state)
                    }
                    "RepeatTrack" => {
                        let state = msg
                            .params
                            .get("state")
                            .and_then(|v| v.as_bool())
                            .ok_or("Missing or invalid repeat_track state parameter")?;
                        Command::RepeatTrack(state)
                    }
                    "Disconnect" => {
                        let pause = msg
                            .params
                            .get("pause")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        Command::Disconnect { pause }
                    }
                    "SetPosition" => {
                        let position = msg
                            .params
                            .get("position")
                            .and_then(|v| v.as_u64())
                            .ok_or("Missing or invalid position parameter")?;
                        Command::SetPosition(
                            position
                                .try_into()
                                .map_err(|_| "Position value out of range")?,
                        )
                    }
                    "SetVolume" => {
                        let volume = msg
                            .params
                            .get("volume")
                            .and_then(|v| v.as_u64())
                            .ok_or("Missing or invalid volume parameter")?;
                        Command::SetVolume(
                            volume.try_into().map_err(|_| "Volume value out of range")?,
                        )
                    }
                    "Activate" => Command::Activate,
                    _ => return Err(format!("Unknown command type: {cmd_type}")),
                };

                let device_id = msg
                    .device_id
                    .ok_or("Device ID is required for this command")?;

                (device_id, command)
            }
        };

        Ok(command)
    }
}

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl CommandResponse {
    pub fn success(message: impl ToString, data: Option<serde_json::Value>) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data,
        }
    }

    pub fn error(message: impl ToString) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}
