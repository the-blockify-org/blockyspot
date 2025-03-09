use crate::commands::{Command, CommandResponse, CommandMessage};
use crate::spotify::SpotifyClient;
use crate::command_manager::CommandManager;
use futures::{FutureExt, StreamExt};
use log::{error, info};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::ws::{Message, WebSocket};
use warp::Filter;

type Clients = Arc<Mutex<HashMap<String, Client>>>;
pub type WsResult<T> = std::result::Result<T, warp::Error>;

#[derive(Debug, serde::Serialize)]
struct ConnectionResponse {
    device_id: String,
    message: String,
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

#[derive(Clone)]
pub struct SpotifyServer {
    clients: Clients,
    command_manager: CommandManager,
}

impl SpotifyServer {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
            command_manager: CommandManager::new(),
        }
    }

    pub async fn start(self, port: u16) {
        let clients = self.clients.clone();

        let ws_route = warp::path("ws")
            .and(warp::ws())
            .and(with_clients(clients.clone()))
            .map(|ws: warp::ws::Ws, clients| {
                ws.on_upgrade(move |socket| Self::handle_client_connection(socket, clients))
            });

        let routes = ws_route.with(warp::cors().allow_any_origin());

        warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    }

    async fn handle_client_connection(ws: WebSocket, clients: Clients) {
        let device_id = Uuid::new_v4().to_string();
        info!("New client connecting with generated device_id: {}", device_id);

        let (ws_sender, mut ws_receiver) = ws.split();
        let (tx, rx) = mpsc::unbounded_channel();

        let rx_stream = UnboundedReceiverStream::new(rx);
        tokio::task::spawn(rx_stream.forward(ws_sender).map(|result| {
            if let Err(e) = result {
                error!("Error sending websocket msg: {}", e);
            }
        }));

        {
            let mut clients = clients.lock().await;
            clients.insert(device_id.clone(), Client::new(tx.clone()));
        }

        let connection_response = ConnectionResponse {
            device_id: device_id.clone(),
            message: "Connected to server. Use this device_id for future commands.".to_string(),
        };
        
        if let Ok(response_json) = serde_json::to_string(&connection_response) {
            if let Err(e) = tx.send(Ok(Message::text(response_json))) {
                error!("Error sending initial device_id: {}", e);
                return;
            }
        }

        let server = SpotifyServer { clients: clients.clone(), command_manager: CommandManager::new() };
        
        while let Some(result) = ws_receiver.next().await {
            let msg = match result {
                Ok(msg) => msg,
                Err(e) => {
                    error!("Error receiving ws message for device_id {}: {}", device_id, e);
                    break;
                }
            };

            if let Ok(text) = msg.to_str() {
                if let Err(e) = Self::process_ws_message(text, &device_id, &tx, &server).await {
                    error!("Error processing message: {}", e);
                    break;
                }
            }
        }

        clients.lock().await.remove(&device_id);
        info!("Client {} disconnected", device_id);
    }

    async fn process_ws_message(
        text: &str,
        device_id: &str,
        tx: &mpsc::UnboundedSender<WsResult<Message>>,
        server: &SpotifyServer,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let command_message: CommandMessage = serde_json::from_str(text)?;
        let (msg_device_id, command) = Command::from_message(command_message)?;

        // Verify the device ID matches
        if msg_device_id != device_id {
            let error_response = CommandResponse::error("Device ID mismatch");
            tx.send(Ok(Message::text(serde_json::to_string(&error_response)?)))?;
            return Ok(());
        }

        let response = {
            let mut clients = server.clients.lock().await;

            match command {
                Command::Connect { token, device_name } => {
                    if let Some(client) = clients.get_mut(device_id) {
                        if client.spotify.is_some() {
                            CommandResponse::error("Device is already connected to Spotify")
                        } else {
                            let mut spotify = SpotifyClient::new();
                            match spotify
                                .initialize(
                                    &token,
                                    device_name.unwrap_or_else(|| format!("Blockyspot {device_id}")),
                                    client.sender.clone(),
                                )
                                .await
                            {
                                Ok(()) => {
                                    client.spotify = Some(spotify);
                                    CommandResponse::success("Connected to Spotify", None)
                                }
                                Err(e) => CommandResponse::error(format!("Failed to connect: {e}")),
                            }
                        }
                    } else {
                        CommandResponse::error("Device not found")
                    }
                }
                cmd => {
                    if let Some(client) = clients.get_mut(device_id) {
                        if let Some(spotify) = &mut client.spotify {
                            server.command_manager.execute(cmd, spotify)
                        } else {
                            CommandResponse::error("Device not connected to Spotify")
                        }
                    } else {
                        CommandResponse::error("Device not found")
                    }
                }
            }
        };

        let response_json = serde_json::to_string(&response)?;
        tx.send(Ok(Message::text(response_json)))?;
        Ok(())
    }
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}