use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub enum Command {
    Connect {
        token: String,
        device_id: String,
        device_name: Option<String>,
    },
    Disconnect {
        device_id: String,
    },
    Play {
        device_id: String,
        track_id: String,
    },
    Pause {
        device_id: String,
    },
    Resume {
        device_id: String,
    },
    Stop {
        device_id: String,
    },
    GetCurrentTrack {
        device_id: String,
    },
}

#[derive(Debug, Serialize)]
pub struct CommandResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl CommandResponse {
    pub fn success(message: &str, data: Option<serde_json::Value>) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data,
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}
