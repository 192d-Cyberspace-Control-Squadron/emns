use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
    Emergency,
}

/// AlertLevel implementation to get as string for logging
impl AlertLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertLevel::Info => "Info",
            AlertLevel::Warning => "Warning",
            AlertLevel::Critical => "Critical",
            AlertLevel::Emergency => "Emergency",
        }
    }
}

/// Alert message sent from server to client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: Uuid,
    pub title: String,
    pub message: String,
    pub level: AlertLevel,
    pub requires_confirmation: bool,
    pub sound_file: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Confirmation sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Confirmation {
    pub alert_id: Uuid,
    pub client_id: String,
    pub confirmed_at: chrono::DateTime<chrono::Utc>,
    pub hostname: String,
    pub username: String,
}

/// Message types for WebSocket communication
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Message {
    Alert { alert: Alert },
    Confirmation { confirmation: Confirmation },
    Heartbeat,
    Register { client_id: String, hostname: String },
}

impl Alert {
    /// Get the sound file path, or default based on level
    pub fn get_sound_file(&self) -> String {
        self.sound_file.clone().unwrap_or_else(|| match self.level {
            AlertLevel::Emergency | AlertLevel::Critical => "alarm_critical.wav".to_string(),
            AlertLevel::Warning => "alarm_warning.wav".to_string(),
            AlertLevel::Info => "notification.wav".to_string(),
        })
    }
}
