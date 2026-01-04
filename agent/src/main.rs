mod audio;
mod client;
mod handler;
mod messages;
mod notification;

use crate::client::WebSocketClient;
use crate::handler::AlertHandler;
use crate::messages::{Alert, Confirmation};
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug)]
pub struct Config {
    pub server_url: String,
    pub client_id: String,
    pub sounds_dir: PathBuf,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let server_url: String =
            std::env::var("SERVER_URL").unwrap_or_else(|_| "ws://localhost:8080/ws".to_string());

        let client_id: String =
            std::env::var("CLIENT_ID").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

        let sounds_dir: PathBuf = std::env::var("SOUNDS_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./sounds"));

        // Create sounds directory if it doesn't exist
        if !sounds_dir.exists() {
            std::fs::create_dir_all(&sounds_dir).context("Failed to create sounds directory")?;
            log::info!("Created sounds directory: {}", sounds_dir.display());
        }

        Ok(Self {
            server_url,
            client_id,
            sounds_dir,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting Notification Agent");

    // Load configuration
    let config: Config = Config::from_env()?;
    log::info!("Configuration loaded:");
    log::info!("  Server URL: {}", config.server_url);
    log::info!("  Client ID: {}", config.client_id);
    log::info!("  Sounds Dir: {}", config.sounds_dir.display());

    // Create channels
    let (alert_tx, mut alert_rx) = mpsc::channel::<Alert>(100);
    let (confirmation_tx, confirmation_rx) = mpsc::channel::<Confirmation>(100);

    // Create alert handler
    let handler: Arc<AlertHandler> = Arc::new(AlertHandler::new(
        config.sounds_dir.clone(),
        confirmation_tx,
        config.client_id.clone(),
    ));

    // Spawn alert processing task
    let handler_clone: Arc<AlertHandler> = handler.clone();
    tokio::spawn(async move {
        while let Some(alert) = alert_rx.recv().await {
            if let Err(e) = handler_clone.handle_alert(alert).await {
                log::error!("Failed to handle alert: {}", e);
            }
        }
    });

    // Create WebSocket client
    let hostname: String = client::get_hostname();
    let ws_client: WebSocketClient = WebSocketClient::new(
        config.server_url.clone(),
        config.client_id.clone(),
        hostname,
    );

    // Show startup notification
    if let Err(e) = notification::show_simple_notification(
        "Notification Agent Started",
        &format!("Connected to: {}", config.server_url),
    ) {
        log::warn!("Failed to show startup notification: {}", e);
    }

    // Run the WebSocket client (this will reconnect on failures)
    ws_client.run(alert_tx, confirmation_rx).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        std::env::remove_var("SERVER_URL");
        std::env::remove_var("CLIENT_ID");
        std::env::remove_var("SOUNDS_DIR");

        let config: Config = Config::from_env().unwrap();
        assert_eq!(config.server_url, "ws://localhost:8080/ws");
        assert!(config.client_id.len() > 0);
        assert_eq!(config.sounds_dir, PathBuf::from("./sounds"));
    }
}
