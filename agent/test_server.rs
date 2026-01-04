/// Example WebSocket server for testing the notification agent
///
/// Run with: cargo run --example test_server
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_tungstenite::{accept_async, tungstenite::Message};
use uuid::Uuid;

type Clients = Arc<Mutex<HashMap<String, tokio::sync::mpsc::Sender<String>>>>;

#[tokio::main]
async fn main() {
    env_logger::init();

    let addr = "127.0.0.1:8080";
    let listener = TcpListener::bind("127.0.0.1:8080")
        .await
        .expect("Failed to bind");
    println!("WebSocket server listening on: {}", addr);

    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));

    // Spawn a task to send periodic test alerts
    let clients_clone = clients.clone();
    tokio::spawn(async move {
        send_test_alerts(clients_clone).await;
    });

    while let Ok((stream, addr)) = listener.accept().await {
        let clients = clients.clone();
        tokio::spawn(handle_connection(stream, addr, clients));
    }
}

async fn handle_connection(stream: TcpStream, addr: SocketAddr, clients: Clients) {
    println!("New connection from: {}", addr);

    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            eprintln!("WebSocket handshake failed: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(100);

    let mut client_id: Option<String> = None;

    // Spawn task to handle outgoing messages
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if let Err(e) = write.send(Message::Text(msg)).await {
                eprintln!("Failed to send message: {}", e);
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received: {}", text);

                // Parse the message
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                    match value["type"].as_str() {
                        Some("register") => {
                            if let Some(id) = value["client_id"].as_str() {
                                client_id = Some(id.to_string());
                                clients.lock().await.insert(id.to_string(), tx.clone());
                                println!("Registered client: {} ({})", id, addr);
                            }
                        }
                        Some("confirmation") => {
                            if let Some(conf) = value.get("confirmation") {
                                println!("Received confirmation for alert: {}", conf["alert_id"]);
                            }
                        }
                        Some("heartbeat") => {
                            println!("Heartbeat from {}", addr);
                        }
                        _ => {
                            println!("Unknown message type");
                        }
                    }
                }
            }
            Ok(Message::Close(_)) => {
                println!("Client {} disconnected", addr);
                break;
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                break;
            }
            _ => {}
        }
    }

    // Remove client on disconnect
    if let Some(id) = client_id {
        clients.lock().await.remove(&id);
        println!("Removed client: {}", id);
    }
}

async fn send_test_alerts(clients: Clients) {
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    let test_alerts = vec![
        (
            "Info Alert",
            "This is an informational message",
            "info",
            false,
        ),
        (
            "Warning Alert",
            "This requires your attention",
            "warning",
            true,
        ),
        (
            "Critical Alert",
            "Critical system event detected!",
            "critical",
            true,
        ),
        (
            "Emergency Alert",
            "IMMEDIATE ACTION REQUIRED",
            "emergency",
            true,
        ),
    ];

    for (i, (title, message, level, requires_confirmation)) in test_alerts.iter().enumerate() {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        let alert = json!({
            "type": "alert",
            "alert": {
                "id": Uuid::new_v4().to_string(),
                "title": title,
                "message": message,
                "level": level,
                "requires_confirmation": requires_confirmation,
                "sound_file": null,
                "timestamp": chrono::Utc::now().to_rfc3339(),
            }
        });

        let alert_str = serde_json::to_string(&alert).unwrap();
        println!("\nSending test alert {}: {}", i + 1, title);

        let clients_lock = clients.lock().await;
        for (client_id, tx) in clients_lock.iter() {
            if let Err(e) = tx.send(alert_str.clone()).await {
                eprintln!("Failed to send alert to {}: {}", client_id, e);
            }
        }
    }

    println!("\nAll test alerts sent. Server will continue running...");
    println!("Press Ctrl+C to stop the server");
}
