use crate::messages::{Alert, AlertLevel};
use anyhow::{Context, Result};
use windows::{
    core::HSTRING,
    Data::Xml::Dom::XmlDocument,
    UI::Notifications::{ToastNotification, ToastNotificationManager},
};

pub struct NotificationManager {
    app_id: String,
}

impl NotificationManager {
    pub fn new(app_id: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
        }
    }

    /// Display a Windows toast notification for the alert
    pub fn show_notification(&self, alert: &Alert) -> Result<()> {
        let xml: XmlDocument = self.create_toast_xml(alert)?;
        let toast: ToastNotification = ToastNotification::CreateToastNotification(&xml)
            .context("Failed to create toast notification")?;

        let notifier: windows::UI::Notifications::ToastNotifier = ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from(
            &self.app_id,
        ))
        .context("Failed to create toast notifier")?;

        notifier
            .Show(&toast)
            .context("Failed to show notification")?;

        log::info!("Displayed notification for alert {}", alert.id);
        Ok(())
    }

    /// Create the XML template for the toast notification
    fn create_toast_xml(&self, alert: &Alert) -> Result<XmlDocument> {
        let (scenario, duration) = match alert.level {
            AlertLevel::Emergency | AlertLevel::Critical => ("urgent", "long"),
            AlertLevel::Warning => ("reminder", "long"),
            AlertLevel::Info => ("default", "short"),
        };

        let icon: &str = match alert.level {
            AlertLevel::Emergency => "⚠️",
            AlertLevel::Critical => "🔴",
            AlertLevel::Warning => "⚡",
            AlertLevel::Info => "ℹ️",
        };

        let confirmation_button: &str = if alert.requires_confirmation {
            r#"<action content="Confirm Receipt" arguments="confirm" activationType="background"/>"#
        } else {
            ""
        };

        let xml_string: String = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<toast scenario="{scenario}" duration="{duration}">
    <visual>
        <binding template="ToastGeneric">
            <text>{icon} {title}</text>
            <text>{message}</text>
            <text>Alert ID: {id}</text>
        </binding>
    </visual>
    <audio src="ms-winsoundevent:Notification.Default" loop="false"/>
    <actions>
        {confirmation_button}
        <action content="Dismiss" arguments="dismiss" activationType="background"/>
    </actions>
</toast>"#,
            scenario = scenario,
            duration = duration,
            icon = icon,
            title = Self::escape_xml(&alert.title),
            message = Self::escape_xml(&alert.message),
            id = alert.id,
            confirmation_button = confirmation_button
        );

        let xml = XmlDocument::new().context("Failed to create XML document")?;
        xml.LoadXml(&HSTRING::from(&xml_string))
            .context("Failed to load XML")?;

        Ok(xml)
    }

    /// Escape XML special characters
    fn escape_xml(s: &str) -> String {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
            .replace('\'', "&apos;")
    }
}

/// Show a simple notification (for testing or status updates)
pub fn show_simple_notification(title: &str, message: &str) -> Result<()> {
    let manager = NotificationManager::new("NotificationAgent");
    let alert = Alert {
        id: uuid::Uuid::new_v4(),
        title: title.to_string(),
        message: message.to_string(),
        level: AlertLevel::Info,
        requires_confirmation: false,
        sound_file: None,
        timestamp: chrono::Utc::now(),
    };
    manager.show_notification(&alert)
}
