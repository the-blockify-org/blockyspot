use crate::command_manager::CommandManager;
use crate::commands::{Command, CommandMessage, CommandResponse};
use crate::spotify::SpotifyClient;
use futures::{FutureExt, StreamExt};
use log::{error, info};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::Filter;
use warp::ws::{Message, WebSocket};

const PROTOCOL_VERSION: &str = "0.1.1";

type Clients = Arc<Mutex<HashMap<String, Client>>>;
pub type WsResult<T> = Result<T, warp::Error>;

#[derive(Debug, serde::Serialize)]
struct ConnectionResponse {
    status: String,
    protocol_version: String,
}

struct Client {
    spotify: Option<SpotifyClient>,
    sender: mpsc::UnboundedSender<WsResult<Message>>,
}

impl Client {
    fn new(sender: mpsc::UnboundedSender<WsResult<Message>>) -> Self {
        Self {
            spotify: None,
            sender,
        }
    }
}

struct ConnectionState {
    devices: HashMap<String, SpotifyClient>,
    sender: mpsc::UnboundedSender<WsResult<Message>>,
}

impl ConnectionState {
    fn new(sender: mpsc::UnboundedSender<WsResult<Message>>) -> Self {
        Self {
            devices: HashMap::new(),
            sender,
        }
    }
}

#[derive(Clone)]
pub struct SpotifyServer {
    command_manager: CommandManager,
}

impl SpotifyServer {
    pub fn new() -> Self {
        Self {
            command_manager: CommandManager::new(),
        }
    }

    pub async fn start(self, port: u16) {
        let server = self.clone();
        let ws_route = warp::path("ws")
            .and(warp::ws())
            .map(move |ws: warp::ws::Ws| {
                let server = server.clone();
                ws.on_upgrade(move |socket| server.handle_client_connection(socket))
            });

        let routes = ws_route.with(warp::cors().allow_any_origin());

        warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    }

    async fn handle_client_connection(self, ws: WebSocket) {
        info!("New client connecting");

        let (ws_sender, mut ws_receiver) = ws.split();
        let (tx, rx) = mpsc::unbounded_channel();

        let rx_stream = UnboundedReceiverStream::new(rx);
        tokio::task::spawn(rx_stream.forward(ws_sender).map(|result| {
            if let Err(e) = result {
                error!("Error sending websocket msg: {}", e);
            }
        }));

        let connection_state = Arc::new(Mutex::new(ConnectionState::new(tx.clone())));

        let connection_response = ConnectionResponse {
            status: "Connected to server".to_string(),
            protocol_version: PROTOCOL_VERSION.to_string(),
        };

        if let Ok(response_json) = serde_json::to_string(&connection_response) {
            if let Err(e) = tx.send(Ok(Message::text(response_json))) {
                error!("Error sending initial connection response: {}", e);
                return;
            }
        }

        while let Some(result) = ws_receiver.next().await {
            let msg = match result {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Error receiving ws message: {}", e);
                    break;
                }
            };

            if let Ok(text) = msg.to_str() {
                if let Err(e) = self
                    .process_ws_message(text, &tx, connection_state.clone())
                    .await
                {
                    error!("Error processing message: {}", e);
                    break;
                }
            }
        }

        info!("Client disconnected");
    }

    async fn process_ws_message(
        &self,
        text: &str,
        tx: &mpsc::UnboundedSender<WsResult<Message>>,
        connection_state: Arc<Mutex<ConnectionState>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let command_message: CommandMessage = match serde_json::from_str(text) {
            Ok(msg) => msg,
            Err(e) => {
                let error_response = CommandResponse::error(format!("Invalid JSON format: {}", e));
                let response_json = serde_json::to_string(&error_response)?;
                tx.send(Ok(Message::text(response_json)))?;
                return Ok(());
            }
        };

        let response = {
            let mut state = connection_state.lock().await;

            match Command::from_message(command_message) {
                Ok((device_id, cmd)) => match cmd {
                    Command::CreateDevice { token, device_name } => {
                        let device_id = Uuid::new_v4().to_string();
                        let mut spotify = SpotifyClient::new();
                        match spotify
                            .initialize(
                                &token,
                                device_name.unwrap_or_else(|| format!("Blockyspot {device_id}")),
                                tx.clone(),
                                device_id.clone(),
                            )
                            .await
                        {
                            Ok(()) => {
                                state.devices.insert(device_id.clone(), spotify);
                                CommandResponse::success(
                                    "Connected to Spotify",
                                    Some(serde_json::json!({ "device_id": device_id })),
                                )
                            }
                            Err(e) => CommandResponse::error(format!("Failed to connect: {e}")),
                        }
                    }
                    cmd => {
                        if let Some(spotify) = state.devices.get_mut(&device_id) {
                            self.command_manager.execute(cmd, spotify)
                        } else {
                            CommandResponse::error("Device not found")
                        }
                    }
                },
                Err(e) => CommandResponse::error(format!("Invalid command: {}", e)),
            }
        };

        let response_json = serde_json::to_string(&response)?;
        tx.send(Ok(Message::text(response_json)))?;
        Ok(())
    }
}
