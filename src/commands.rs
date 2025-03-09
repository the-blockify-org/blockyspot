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
    Disconnect,
    Play {
        track_id: String,
    },
    Pause,
    Resume,
    Stop,
    GetCurrentTrack,
}

impl Command {
    pub fn from_message(msg: CommandMessage) -> Result<(String, Command), String> {
        let command = match msg.command_type.as_str() {
            "CreateDevice" => {
                let token = msg.params.get("token")
                    .and_then(|v| v.as_str())
                    .ok_or("Missing token parameter")?
                    .to_string();
                
                let device_name = msg.params.get("device_name")
                    .and_then(|v| v.as_str())
                    .map(String::from);
                
                (String::new(), Command::CreateDevice { token, device_name })
            },
            cmd_type => {
                let device_id = msg.device_id
                    .ok_or("Device ID is required for this command")?;
                
                let command = match cmd_type {
                    "play" => {
                        let track_id = msg.params.get("track_id")
                            .and_then(|v| v.as_str())
                            .ok_or("Missing track_id parameter")?
                            .to_string();
                        
                        Command::Play { track_id }
                    },
                    "pause" => Command::Pause,
                    "resume" => Command::Resume,
                    "stop" => Command::Stop,
                    "get_current_track" => Command::GetCurrentTrack,
                    _ => return Err(format!("Unknown command type: {}", cmd_type))
                };
                
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
