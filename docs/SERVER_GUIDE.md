# Server Implementation Guide

This guide helps you build a notification server that works with the Windows Notification Agent.

## Protocol Overview

The agent and server communicate via WebSocket using JSON messages. All messages have a `type` field that determines the message structure.

## Message Types

### 1. Client → Server: Registration

Sent when the client first connects to identify itself.

```json
{
  "type": "register",
  "client_id": "workstation-001",
  "hostname": "WIN-DESKTOP"
}
```

**Fields:**

- `client_id`: Unique identifier for this client
- `hostname`: Computer hostname

**Server Action:** Track this client for sending alerts.

### 2. Server → Client: Alert

Sent to notify the client of an event.

```json
{
  "type": "alert",
  "alert": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "title": "Network Outage",
    "message": "Building 3 network is down. Estimated repair: 2 hours",
    "level": "critical",
    "requires_confirmation": true,
    "sound_file": "alarm_critical.wav",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

**Alert Fields:**

- `id`: UUID for this alert (used for confirmation tracking)
- `title`: Brief alert title (shown prominently)
- `message`: Detailed alert message
- `level`: One of: `"info"`, `"warning"`, `"critical"`, `"emergency"`
- `requires_confirmation`: Boolean - if true, client must confirm receipt
- `sound_file`: Optional WAV filename (null for default based on level)
- `timestamp`: ISO 8601 timestamp

**Alert Levels:**

- `info`: Standard notification (blue)
- `warning`: Important notification (yellow)
- `critical`: Serious issue (red)
- `emergency`: Highest priority (urgent display mode)

### 3. Client → Server: Confirmation

Sent when user confirms receipt of an alert.

```json
{
  "type": "confirmation",
  "confirmation": {
    "alert_id": "123e4567-e89b-12d3-a456-426614174000",
    "client_id": "workstation-001",
    "confirmed_at": "2024-01-15T10:35:00Z",
    "hostname": "WIN-DESKTOP",
    "username": "jdoe"
  }
}
```

**Confirmation Fields:**

- `alert_id`: UUID of the alert being confirmed
- `client_id`: Client identifier
- `confirmed_at`: ISO 8601 timestamp of confirmation
- `hostname`: Computer hostname
- `username`: Windows username who confirmed

**Server Action:** Record confirmation, stop tracking unconfirmed alert.

### 4. Bidirectional: Heartbeat

Sent periodically (every 30 seconds) to maintain connection.

```json
{
  "type": "heartbeat"
}
```

**Purpose:** Detect dead connections, keep NAT mappings alive.

## Server Implementation Checklist

### Basic Requirements

- [ ] WebSocket server implementation
- [ ] Parse JSON messages with `type` field
- [ ] Handle client registration
- [ ] Send alerts to registered clients
- [ ] Receive and log confirmations
- [ ] Respond to heartbeats

### Recommended Features

- [ ] Track connected clients
- [ ] Persist unconfirmed alerts
- [ ] Implement alert queuing for offline clients
- [ ] Add authentication (tokens, certificates)
- [ ] Use TLS (wss://)
- [ ] Log all confirmations to database
- [ ] Implement alert expiration
- [ ] Support alert priorities/routing

### Production Considerations

- [ ] Rate limiting
- [ ] Connection timeouts
- [ ] Reconnection backoff
- [ ] Message persistence
- [ ] Monitoring/metrics
- [ ] Alert deduplication
- [ ] Client heartbeat monitoring

## Example Server (Rust)

See `examples/test_server.rs` for a basic implementation.

## Example Server (Python)

```python
import asyncio
import json
import uuid
from datetime import datetime, timezone
from websockets.server import serve

clients = {}

async def handler(websocket):
    client_id = None
    try:
        async for message in websocket:
            data = json.loads(message)
            
            if data["type"] == "register":
                client_id = data["client_id"]
                clients[client_id] = websocket
                print(f"Registered: {client_id}")
                
            elif data["type"] == "confirmation":
                conf = data["confirmation"]
                print(f"Confirmed: {conf['alert_id']} by {conf['username']}")
                
            elif data["type"] == "heartbeat":
                print(f"Heartbeat from {client_id}")
                
    finally:
        if client_id and client_id in clients:
            del clients[client_id]
            print(f"Disconnected: {client_id}")

async def send_alert(client_id, title, message, level="info", requires_confirmation=False):
    """Send an alert to a specific client"""
    if client_id not in clients:
        print(f"Client {client_id} not connected")
        return
        
    alert = {
        "type": "alert",
        "alert": {
            "id": str(uuid.uuid4()),
            "title": title,
            "message": message,
            "level": level,
            "requires_confirmation": requires_confirmation,
            "sound_file": None,
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
    }
    
    await clients[client_id].send(json.dumps(alert))

async def broadcast_alert(title, message, level="info", requires_confirmation=False):
    """Broadcast an alert to all connected clients"""
    alert = {
        "type": "alert",
        "alert": {
            "id": str(uuid.uuid4()),
            "title": title,
            "message": message,
            "level": level,
            "requires_confirmation": requires_confirmation,
            "sound_file": None,
            "timestamp": datetime.now(timezone.utc).isoformat()
        }
    }
    
    alert_json = json.dumps(alert)
    disconnected = []
    
    for client_id, ws in clients.items():
        try:
            await ws.send(alert_json)
        except:
            disconnected.append(client_id)
    
    # Remove disconnected clients
    for client_id in disconnected:
        del clients[client_id]

async def main():
    async with serve(handler, "localhost", 8080):
        print("Server started on ws://localhost:8080")
        
        # Example: Send test alerts
        await asyncio.sleep(10)
        await broadcast_alert(
            "Test Alert",
            "This is a test notification",
            level="warning",
            requires_confirmation=True
        )
        
        await asyncio.Future()  # Run forever

if __name__ == "__main__":
    asyncio.run(main())
```

## Example Server (Node.js)

```javascript
const WebSocket = require('ws');
const { v4: uuidv4 } = require('uuid');

const wss = new WebSocket.Server({ port: 8080 });
const clients = new Map();

wss.on('connection', (ws) => {
  let clientId = null;

  ws.on('message', (data) => {
    const msg = JSON.parse(data);

    switch (msg.type) {
      case 'register':
        clientId = msg.client_id;
        clients.set(clientId, ws);
        console.log(`Registered: ${clientId}`);
        break;

      case 'confirmation':
        const conf = msg.confirmation;
        console.log(`Confirmed: ${conf.alert_id} by ${conf.username}`);
        break;

      case 'heartbeat':
        console.log(`Heartbeat from ${clientId}`);
        break;
    }
  });

  ws.on('close', () => {
    if (clientId) {
      clients.delete(clientId);
      console.log(`Disconnected: ${clientId}`);
    }
  });
});

function sendAlert(clientId, title, message, level = 'info', requiresConfirmation = false) {
  const ws = clients.get(clientId);
  if (!ws) {
    console.log(`Client ${clientId} not connected`);
    return;
  }

  const alert = {
    type: 'alert',
    alert: {
      id: uuidv4(),
      title,
      message,
      level,
      requires_confirmation: requiresConfirmation,
      sound_file: null,
      timestamp: new Date().toISOString()
    }
  };

  ws.send(JSON.stringify(alert));
}

function broadcastAlert(title, message, level = 'info', requiresConfirmation = false) {
  const alert = {
    type: 'alert',
    alert: {
      id: uuidv4(),
      title,
      message,
      level,
      requires_confirmation: requiresConfirmation,
      sound_file: null,
      timestamp: new Date().toISOString()
    }
  };

  const alertJson = JSON.stringify(alert);

  clients.forEach((ws, clientId) => {
    try {
      ws.send(alertJson);
    } catch (err) {
      console.error(`Failed to send to ${clientId}:`, err);
      clients.delete(clientId);
    }
  });
}

console.log('Server started on ws://localhost:8080');

// Example: Send a test alert after 10 seconds
setTimeout(() => {
  broadcastAlert(
    'Test Alert',
    'This is a test notification',
    'warning',
    true
  );
}, 10000);
```

## Alert Design Best Practices

### Title Guidelines

- Keep under 60 characters
- Front-load important information
- Use action verbs for urgency

**Good:** "Network Down - Building 3"  
**Bad:** "There seems to be an issue with the network connectivity in Building 3"

### Message Guidelines

- Provide context and impact
- Include timeline if known
- Suggest actions if applicable
- Keep under 200 characters for best display

**Example:**

```
Title: "Database Backup Failed"
Message: "Nightly backup failed at 02:15. Data since yesterday at risk. Attempting manual backup now."
```

### Level Selection

- **Info**: Status updates, completions, non-urgent FYI
- **Warning**: Issues that need attention soon but aren't critical
- **Critical**: Problems affecting operations that need immediate attention
- **Emergency**: Safety issues, security breaches, complete outages

### Confirmation Strategy

Require confirmation for:

- Critical/Emergency alerts
- Compliance-related notifications
- Alerts requiring acknowledgment for audit trails
- Multi-step procedures where confirmation ensures next steps occur

Don't require confirmation for:

- Informational updates
- Routine status messages
- High-frequency alerts

## Integration Examples

### Alert from Monitoring System

```python
# Integrate with Prometheus, Nagios, etc.
async def on_monitoring_alert(alert_data):
    level = "info"
    if alert_data["severity"] == "critical":
        level = "critical"
    elif alert_data["severity"] == "warning":
        level = "warning"
        
    await broadcast_alert(
        title=alert_data["title"],
        message=alert_data["description"],
        level=level,
        requires_confirmation=(level in ["critical", "emergency"])
    )
```

### Alert from Security System

```python
# Integrate with SIEM, IDS, etc.
async def on_security_event(event):
    await broadcast_alert(
        title=f"Security Alert: {event['type']}",
        message=f"{event['description']} - Source: {event['source_ip']}",
        level="emergency",
        requires_confirmation=True
    )
```

## Testing Your Server

1. Start your server
2. Run the test client: `cargo run`
3. Send a test alert from your server
4. Verify notification appears
5. Check confirmation is received

## Security Recommendations

1. **Use TLS**: Always use `wss://` in production
2. **Authenticate Clients**: Implement token or certificate-based auth
3. **Validate Input**: Sanitize all incoming messages
4. **Rate Limit**: Prevent alert flooding
5. **Encrypt Sensitive Data**: Don't send credentials in plain text
6. **Audit Trail**: Log all alerts and confirmations
7. **Access Control**: Restrict who can send alerts
