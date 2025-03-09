use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Command {
    Connect {
        token: String,
        device_id: String,
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
