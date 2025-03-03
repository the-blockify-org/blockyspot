use anyhow::Result;
use librespot::connect::{ConnectConfig, LoadRequest, LoadRequestOptions, Spirc};
use librespot::core::authentication::Credentials;
use librespot::core::cache::Cache;
use librespot::core::config::SessionConfig;
use librespot::core::session::Session;
use librespot::playback::{
    audio_backend,
    config::{AudioFormat, PlayerConfig},
    mixer,
    mixer::MixerConfig,
    player::Player,
};
use std::sync::Arc;

const CACHE: &str = ".cache";
const CACHE_FILES: &str = ".cache/files";

pub struct SpotifyClient {
    session: Option<Session>,
    player: Option<Arc<Player>>,
    spirc: Option<Arc<Spirc>>,
    spirc_task: Option<tokio::task::JoinHandle<()>>,
    device_name: String,
}

impl SpotifyClient {
    pub fn new() -> Self {
        Self {
            session: None,
            player: None,
            spirc: None,
            spirc_task: None,
            device_name: String::new(),
        }
    }

    pub async fn initialize(&mut self, token: String, device_name: String) -> Result<()> {
        self.device_name = device_name.clone();

        let connect_config = ConnectConfig {
            name: device_name,
            ..Default::default()
        };
        let session_config = SessionConfig::default();
        let player_config = PlayerConfig::default();
        let audio_format = AudioFormat::default();
        let mixer_config = MixerConfig::default();

        let sink_builder = audio_backend::find(None).unwrap();
        let mixer_builder = mixer::find(None).unwrap();

        // Create cache
        let cache = Cache::new(Some(CACHE), Some(CACHE), Some(CACHE_FILES), None)?;

        // Create credentials from token
        let credentials = Credentials::with_access_token(&token);

        let session = Session::new(session_config, Some(cache));
        let mixer = mixer_builder(mixer_config);

        let player = Player::new(
            player_config,
            session.clone(),
            mixer.get_soft_volume(),
            move || sink_builder(None, audio_format),
        );

        let (spirc, spirc_task) = Spirc::new(
            connect_config,
            session.clone(),
            credentials,
            player.clone(),
            mixer,
        )
        .await?;

        let spirc = Arc::new(spirc);

        // Queue the commands
        spirc.activate()?;

        self.session = Some(session);
        self.player = Some(player);
        self.spirc = Some(spirc);
        self.spirc_task = Some(tokio::spawn(spirc_task));

        Ok(())
    }

    pub fn play_track(&mut self, track_id: String) -> Result<()> {
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

    pub fn pause(&mut self) -> Result<()> {
        if let Some(spirc) = &self.spirc {
            spirc.pause()?;
            Ok(())
        } else {
            anyhow::bail!("Spotify Connect device not initialized")
        }
    }

    pub fn resume(&mut self) -> Result<()> {
        if let Some(spirc) = &self.spirc {
            spirc.play()?;
            Ok(())
        } else {
            anyhow::bail!("Spotify Connect device not initialized")
        }
    }

    pub fn stop_playback(&mut self) -> Result<()> {
        if let Some(spirc) = &self.spirc {
            spirc.shutdown()?;
            Ok(())
        } else {
            anyhow::bail!("Spotify Connect device not initialized")
        }
    }

    // TODO: Implement methods for:
    // - Getting current track information
    // - Managing the audio stream
    // - Handling player events
}
