use crate::messages::{Alert, Confirmation, Message};
use anyhow::{Context, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};

pub struct WebSocketClient {
    server_url: String,
    client_id: String,
    hostname: String,
}

impl WebSocketClient {
    pub fn new(server_url: String, client_id: String, hostname: String) -> Self {
        Self {
            server_url,
            client_id,
            hostname,
        }
    }

    /// Connect to the server and handle messages
    pub async fn run(
        &self,
        alert_tx: mpsc::Sender<Alert>,
        mut confirmation_rx: mpsc::Receiver<Confirmation>,
    ) -> Result<()> {
        loop {
            match self
                .connect_and_handle(alert_tx.clone(), &mut confirmation_rx)
                .await
            {
                Ok(_) => {
                    log::info!("WebSocket connection closed normally");
                }
                Err(e) => {
                    log::error!("WebSocket error: {}", e);
                }
            }

            log::info!("Reconnecting in 5 seconds...");
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn connect_and_handle(
        &self,
        alert_tx: mpsc::Sender<Alert>,
        confirmation_rx: &mut mpsc::Receiver<Confirmation>,
    ) -> Result<()> {
        log::info!("Connecting to {}", self.server_url);

        let (ws_stream, _) = connect_async(&self.server_url)
            .await
            .context("Failed to connect to WebSocket server")?;

        log::info!("Connected to server");

        let (mut write, mut read) = ws_stream.split();

        // Send registration message
        let register_msg: Message = Message::Register {
            client_id: self.client_id.clone(),
            hostname: self.hostname.clone(),
        };
        let json: String = serde_json::to_string(&register_msg)?;
        write.send(WsMessage::Text(json)).await?;
        log::info!("Sent registration message");

        // Heartbeat timer
        let mut heartbeat: tokio::time::Interval = interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                // Handle incoming messages from server
                msg = read.next() => {
                    match msg {
                        Some(Ok(WsMessage::Text(text))) => {
                            self.handle_server_message(&text, &alert_tx).await?;
                        }
                        Some(Ok(WsMessage::Close(_))) => {
                            log::info!("Server closed connection");
                            break;
                        }
                        Some(Err(e)) => {
                            return Err(e.into());
                        }
                        None => {
                            log::info!("Connection closed");
                            break;
                        }
                        _ => {}
                    }
                }

                // Send confirmations to server
                Some(confirmation) = confirmation_rx.recv() => {
                    let msg = Message::Confirmation { confirmation };
                    let json = serde_json::to_string(&msg)?;
                    write.send(WsMessage::Text(json)).await?;
                    log::info!("Sent confirmation to server");
                }

                // Send heartbeat
                _ = heartbeat.tick() => {
                    let msg = Message::Heartbeat;
                    let json = serde_json::to_string(&msg)?;
                    write.send(WsMessage::Text(json)).await?;
                    log::debug!("Sent heartbeat");
                }
            }
        }

        Ok(())
    }

    async fn handle_server_message(
        &self,
        text: &str,
        alert_tx: &mpsc::Sender<Alert>,
    ) -> Result<()> {
        let message: Message =
            serde_json::from_str(text).context("Failed to parse server message")?;

        match message {
            Message::Alert { alert } => {
                log::info!("Received alert: {} - {}", alert.id, alert.title);
                alert_tx
                    .send(alert)
                    .await
                    .context("Failed to send alert to handler")?;
            }
            Message::Heartbeat => {
                log::debug!("Received heartbeat from server");
            }
            _ => {
                log::warn!("Unexpected message type from server");
            }
        }

        Ok(())
    }
}

/// Get the hostname of the machine
pub fn get_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Get the current username
pub fn get_username() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "unknown".to_string())
}
