use anyhow::Result;
use librespot::connect::{ConnectConfig, Spirc};
use librespot::core::authentication::Credentials;
use librespot::core::cache::Cache;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::playback::{
    config::{AudioFormat, PlayerConfig},
    mixer,
    mixer::MixerConfig,
    player::Player,
    player::SinkStatus,
    player::PlayerEvent,
};
use std::sync::Arc;
use tokio::sync::mpsc;
use warp::ws::Message;
use crate::server::WsResult;
use tokio::task;
use crate::ws_sink::create_ws_sink;

const CACHE: &str = ".cache";
const CACHE_FILES: &str = ".cache/files";

macro_rules! spirc_call {
    ($self:expr, $method:ident) => {
        if let Some(spirc) = &$self.spirc {
            spirc.$method().map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", stringify!($method), e))
        } else {
            anyhow::bail!("Spotify Connect device not initialized")
        }
    };
    ($self:expr, $method:ident, $($arg:expr),+) => {
        if let Some(spirc) = &$self.spirc {
            spirc.$method($($arg),+).map_err(|e| anyhow::anyhow!("Failed to execute {}: {}", stringify!($method), e))
        } else {
            anyhow::bail!("Spotify Connect device not initialized")
        }
    };
}

#[derive(Default)]
pub struct SpotifyClient {
    session: Option<Session>,
    player: Option<Arc<Player>>,
    spirc: Option<Arc<Spirc>>,
    spirc_task: Option<tokio::task::JoinHandle<()>>,
    device_name: String,
    device_id: String,
    ws_sender: Option<mpsc::UnboundedSender<WsResult<Message>>>,
    player_event_task: Option<task::JoinHandle<()>>,
}

impl SpotifyClient {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn initialize(
        &mut self,
        token: impl Into<String>,
        device_name: String,
        ws_sender: mpsc::UnboundedSender<WsResult<Message>>,
        device_id: String,
    ) -> Result<()> {
        self.device_name = device_name.clone();
        let ws_sender_clone = ws_sender.clone();
        self.ws_sender = Some(ws_sender);
        self.device_id = device_id;

        let connect_config = ConnectConfig {
            name: device_name,
            ..Default::default()
        };
        let session_config = SessionConfig::default();
        let player_config = PlayerConfig::default();
        let audio_format = AudioFormat::default();
        let mixer_config = MixerConfig::default();

        let device_id_clone = self.device_id.clone();
        let sink_builder = move || {
            create_ws_sink(ws_sender_clone.clone(), audio_format, device_id_clone)
        };
        let mixer_builder = mixer::find(None).unwrap();

        let cache = Cache::new(Some(CACHE), Some(CACHE), Some(CACHE_FILES), None)?;

        let credentials = Credentials::with_access_token(token);

        let session = Session::new(session_config, Some(cache));
        let mixer = mixer_builder(mixer_config)?;

        let player = Player::new(
            player_config,
            session.clone(),
            mixer.get_soft_volume(),
            sink_builder,
        );

        // Set up sink event callbacks
        let ws_sender_clone = self.ws_sender.clone();
        let device_id_clone = self.device_id.clone();
        player.set_sink_event_callback(Some(Box::new(move |event: SinkStatus| {
            if let Some(sender) = &ws_sender_clone {
                let event_json = serde_json::json!({
                    "type": "sink_event",
                    "device_id": device_id_clone,
                    "data": {
                        "status": format!("{:?}", event),
                    }
                });
                
                if let Ok(msg) = serde_json::to_string(&event_json) {
                    let _ = sender.send(Ok(Message::text(msg)));
                }
            }
        })));

        // Set up player event channel
        let mut event_channel = player.get_player_event_channel();
        let ws_sender_clone = self.ws_sender.clone();
        let  device_id_clone = self.device_id.clone();
        
        // Spawn a task to handle player events
        let player_event_task = tokio::spawn(async move {
            while let Some(event) = event_channel.recv().await {
                if let Some(sender) = &ws_sender_clone {
                    let event_json = serde_json::json!({
                        "type": "player_event",
                        "device_id": device_id_clone,
                        "data": {
                            "event_type": format!("{:?}", event),
                            "details": match &event {
                                PlayerEvent::Playing { play_request_id, track_id, position_ms } => {
                                    serde_json::json!({
                                        "play_request_id": play_request_id,
                                        "track_id": track_id.to_string(),
                                        "position_ms": position_ms
                                    })
                                },
                                PlayerEvent::Paused { play_request_id, track_id, position_ms } => {
                                    serde_json::json!({
                                        "play_request_id": play_request_id,
                                        "track_id": track_id.to_string(),
                                        "position_ms": position_ms
                                    })
                                },
                                PlayerEvent::Stopped { play_request_id, track_id } => {
                                    serde_json::json!({
                                        "play_request_id": play_request_id,
                                        "track_id": track_id.to_string()
                                    })
                                },
                                PlayerEvent::Loading { play_request_id, track_id, position_ms } => {
                                    serde_json::json!({
                                        "play_request_id": play_request_id,
                                        "track_id": track_id.to_string(),
                                        "position_ms": position_ms
                                    })
                                },
                                PlayerEvent::EndOfTrack { play_request_id, track_id } => {
                                    serde_json::json!({
                                        "play_request_id": play_request_id,
                                        "track_id": track_id.to_string()
                                    })
                                },
                                _ => serde_json::json!(null)
                            }
                        }
                    });

                    if let Ok(msg) = serde_json::to_string(&event_json) {
                        let _ = sender.send(Ok(Message::text(msg)));
                    }
                }
            }
        });

        self.player_event_task = Some(player_event_task);

        let (spirc, spirc_task) = Spirc::new(
            connect_config,
            session.clone(),
            credentials,
            player.clone(),
            mixer.clone(),
        )
        .await?;

        let spirc = Arc::new(spirc);

        self.session = Some(session);
        self.player = Some(player);
        self.spirc = Some(spirc);
        self.spirc_task = Some(tokio::spawn(spirc_task));

        Ok(())
    }

    // Direct Spirc wrapper methods
    pub fn shutdown(&self) -> Result<()> {
        spirc_call!(self, shutdown)
    }

    pub fn play(&self) -> Result<()> {
        spirc_call!(self, play)
    }

    pub fn play_pause(&self) -> Result<()> {
        spirc_call!(self, play_pause)
    }

    pub fn pause(&self) -> Result<()> {
        spirc_call!(self, pause)
    }

    pub fn prev(&self) -> Result<()> {
        spirc_call!(self, prev)
    }

    pub fn next(&self) -> Result<()> {
        spirc_call!(self, next)
    }

    pub fn volume_up(&self) -> Result<()> {
        spirc_call!(self, volume_up)
    }

    pub fn volume_down(&self) -> Result<()> {
        spirc_call!(self, volume_down)
    }

    pub fn shuffle(&self, shuffle: bool) -> Result<()> {
        spirc_call!(self, shuffle, shuffle)
    }

    pub fn repeat(&self, repeat: bool) -> Result<()> {
        spirc_call!(self, repeat, repeat)
    }

    pub fn repeat_track(&self, repeat: bool) -> Result<()> {
        spirc_call!(self, repeat_track, repeat)
    }

    pub fn set_volume(&self, volume: u16) -> Result<()> {
        spirc_call!(self, set_volume, volume)
    }

    pub fn set_position_ms(&self, position_ms: u32) -> Result<()> {
        spirc_call!(self, set_position_ms, position_ms)
    }

    pub fn disconnect(&self, pause: bool) -> Result<()> {
        spirc_call!(self, disconnect, pause)
    }

    pub fn activate(&self) -> Result<()> {
        spirc_call!(self, activate)
    }
}
