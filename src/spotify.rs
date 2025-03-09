use anyhow::Result;
use librespot::connect::{ConnectConfig, LoadRequest, LoadRequestOptions, Spirc};
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
use serde_json;
use tokio::task;
use crate::ws_sink::create_ws_sink;

const CACHE: &str = ".cache";
const CACHE_FILES: &str = ".cache/files";

#[derive(Default)]
pub struct SpotifyClient {
    session: Option<Session>,
    player: Option<Arc<Player>>,
    spirc: Option<Arc<Spirc>>,
    spirc_task: Option<tokio::task::JoinHandle<()>>,
    device_name: String,
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
    ) -> Result<()> {
        self.device_name = device_name.clone();
        let ws_sender_clone = ws_sender.clone();
        self.ws_sender = Some(ws_sender);

        let connect_config = ConnectConfig {
            name: device_name,
            ..Default::default()
        };
        let session_config = SessionConfig::default();
        let player_config = PlayerConfig::default();
        let audio_format = AudioFormat::default();
        let mixer_config = MixerConfig::default();

        let sink_builder = move || {
            create_ws_sink(ws_sender_clone.clone(), audio_format)
        };
        let mixer_builder = mixer::find(None).unwrap();

        let cache = Cache::new(Some(CACHE), Some(CACHE), Some(CACHE_FILES), None)?;

        let credentials = Credentials::with_access_token(token);

        let session = Session::new(session_config, Some(cache));
        let mixer = mixer_builder(mixer_config);

        let player = Player::new(
            player_config,
            session.clone(),
            mixer.get_soft_volume(),
            sink_builder,
        );

        // Set up sink event callbacks
        let ws_sender_clone = self.ws_sender.clone();
        player.set_sink_event_callback(Some(Box::new(move |event: SinkStatus| {
            if let Some(sender) = &ws_sender_clone {
                let event_json = serde_json::json!({
                    "type": "sink_event",
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
        
        // Spawn a task to handle player events
        let player_event_task = tokio::spawn(async move {
            while let Some(event) = event_channel.recv().await {
                if let Some(sender) = &ws_sender_clone {
                    let event_json = serde_json::json!({
                        "type": "player_event",
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
            mixer,
        )
        .await?;

        let spirc = Arc::new(spirc);

        self.session = Some(session);
        self.player = Some(player);
        self.spirc = Some(spirc);
        self.spirc_task = Some(tokio::spawn(spirc_task));

        Ok(())
    }

    pub fn play_track(&self, track_id: impl AsRef<str>) -> Result<()> {
        let track_id = track_id.as_ref();

        if let Some(spirc) = &self.spirc {
            let options = LoadRequestOptions::default();
            let request =
                LoadRequest::from_context_uri(format!("spotify:track:{track_id}"), options);
            spirc.activate()?;
            spirc.load(request)?;
            spirc.play()?;
            Ok(())
        } else {
            anyhow::bail!("Spotify Connect device not initialized")
        }
    }

    pub fn pause(&self) -> Result<()> {
        if let Some(spirc) = &self.spirc {
            spirc.pause()?;
            Ok(())
        } else {
            anyhow::bail!("Spotify Connect device not initialized")
        }
    }

    pub fn resume(&self) -> Result<()> {
        if let Some(spirc) = &self.spirc {
            spirc.play()?;
            Ok(())
        } else {
            anyhow::bail!("Spotify Connect device not initialized")
        }
    }

    pub fn stop_playback(&self) -> Result<()> {
        if let Some(spirc) = &self.spirc {
            spirc.shutdown()?;
            // Cancel the player event task when stopping playback
            if let Some(task) = &self.player_event_task {
                task.abort();
            }
            Ok(())
        } else {
            anyhow::bail!("Spotify Connect device not initialized")
        }
    }

    // Helper method to send WebSocket messages
    fn send_ws_message(&self, message: impl serde::Serialize) -> Result<()> {
        if let Some(sender) = &self.ws_sender {
            let msg_str = serde_json::to_string(&message)?;
            sender.send(Ok(Message::text(msg_str)))
                .map_err(|e| anyhow::anyhow!("Failed to send WebSocket message: {}", e))?;
        }
        Ok(())
    }

    // TODO: Implement methods for:
    // - Getting current track information
    // - Managing the audio stream
    // - Handling player events
}
