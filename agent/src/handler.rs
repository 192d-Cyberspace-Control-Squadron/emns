use crate::audio::AudioPlayer;
use crate::client::{get_hostname, get_username};
use crate::messages::{Alert, Confirmation};
use crate::notification::NotificationManager;
use anyhow::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};

pub struct AlertHandler {
    notification_manager: NotificationManager,
    audio_player: AudioPlayer,
    pending_confirmations: Arc<Mutex<HashMap<uuid::Uuid, Alert>>>,
    confirmation_tx: mpsc::Sender<Confirmation>,
    client_id: String,
}

impl AlertHandler {
    pub fn new(
        sounds_dir: PathBuf,
        confirmation_tx: mpsc::Sender<Confirmation>,
        client_id: String,
    ) -> Self {
        Self {
            notification_manager: NotificationManager::new("NotificationAgent"),
            audio_player: AudioPlayer::new(sounds_dir),
            pending_confirmations: Arc::new(Mutex::new(HashMap::new())),
            confirmation_tx,
            client_id,
        }
    }

    /// Handle an incoming alert
    pub async fn handle_alert(&self, alert: Alert) -> Result<()> {
        log::info!(
            "Processing alert {}: {} - {}",
            alert.id,
            alert.level.as_str(),
            alert.title
        );

        // Play sound (async, non-blocking)
        let sound_file = alert.get_sound_file();
        self.audio_player.play_sound_async(sound_file);

        // Show notification
        if let Err(e) = self.notification_manager.show_notification(&alert) {
            log::error!("Failed to show notification: {}", e);
        }

        // Track for confirmation if required
        if alert.requires_confirmation {
            let alert_id = alert.id;
            self.pending_confirmations
                .lock()
                .await
                .insert(alert_id, alert.clone());

            // Auto-confirm after timeout (e.g., 5 minutes)
            let pending = self.pending_confirmations.clone();
            let tx = self.confirmation_tx.clone();
            let client_id = self.client_id.clone();

            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;

                let mut pending = pending.lock().await;
                if pending.contains_key(&alert_id) {
                    log::warn!(
                        "Alert {} not confirmed within timeout, auto-confirming",
                        alert_id
                    );
                    pending.remove(&alert_id);

                    let confirmation = Confirmation {
                        alert_id,
                        client_id,
                        confirmed_at: chrono::Utc::now(),
                        hostname: get_hostname(),
                        username: get_username(),
                    };

                    let _ = tx.send(confirmation).await;
                }
            });
        }

        Ok(())
    }

    /// Manually confirm an alert
    pub async fn confirm_alert(&self, alert_id: uuid::Uuid) -> Result<()> {
        let mut pending = self.pending_confirmations.lock().await;

        if pending.remove(&alert_id).is_some() {
            log::info!("Alert {} confirmed by user", alert_id);

            let confirmation = Confirmation {
                alert_id,
                client_id: self.client_id.clone(),
                confirmed_at: chrono::Utc::now(),
                hostname: get_hostname(),
                username: get_username(),
            };

            self.confirmation_tx
                .send(confirmation)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to send confirmation: {}", e))?;

            Ok(())
        } else {
            log::warn!("Alert {} not found in pending confirmations", alert_id);
            Ok(())
        }
    }

    /// Get pending confirmations count
    pub async fn pending_count(&self) -> usize {
        self.pending_confirmations.lock().await.len()
    }

    /// Get all pending alert IDs
    pub async fn get_pending_alerts(&self) -> Vec<uuid::Uuid> {
        self.pending_confirmations
            .lock()
            .await
            .keys()
            .copied()
            .collect()
    }
}
