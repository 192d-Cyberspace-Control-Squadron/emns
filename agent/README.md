# Emergency Management Notification System Agent

A robust Windows notification agent built in Rust that receives alerts from a server, displays native toast notifications, plays audio alerts, and confirms receipt.

## Features

- **WebSocket Communication**: Real-time connection to alert server with automatic reconnection
- **Windows Toast Notifications**: Native Windows 10/11 toast notifications with custom severity levels
- **Audio Alerts**: Plays WAV files for different alert levels with fallback to system beeps
- **Confirmation Tracking**: Tracks and confirms alert receipt back to server
- **Auto-reconnect**: Automatically reconnects to server on connection loss
- **Heartbeat**: Maintains connection health with periodic heartbeats

## Alert Severity Levels

- **Info**: Standard notifications (blue icon)
- **Warning**: Important notifications requiring attention (yellow icon)
- **Critical**: Serious issues requiring immediate attention (red icon)
- **Emergency**: Highest priority alerts (urgent scenario with extended display)

## Requirements

- Windows 10/11
- Rust toolchain (1.70+)
- Audio output device

## Installation

```bash
# Clone the repository
git clone <repository-url>
cd notification-agent

# Build the application
cargo build --release

# The executable will be in target/release/notification-agent.exe
```

## Configuration

The agent is configured via environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `SERVER_URL` | WebSocket server URL | `ws://localhost:8080/ws` |
| `CLIENT_ID` | Unique client identifier | Auto-generated UUID |
| `SOUNDS_DIR` | Directory containing sound files | `./sounds` |

### Example

```powershell
# PowerShell
$env:SERVER_URL = "ws://alerts.example.com:8080/ws"
$env:CLIENT_ID = "workstation-01"
$env:SOUNDS_DIR = "C:\AlertSounds"

.\target\release\notification-agent.exe
```

```cmd
# Command Prompt
set SERVER_URL=ws://alerts.example.com:8080/ws
set CLIENT_ID=workstation-01
set SOUNDS_DIR=C:\AlertSounds

.\target\release\notification-agent.exe
```

## Sound Files

Place WAV files in the `sounds` directory. Default filenames:

- `alarm_critical.wav` - For Critical/Emergency alerts
- `alarm_warning.wav` - For Warning alerts
- `notification.wav` - For Info alerts

Custom sound files can be specified per-alert in the server message.

## Protocol

### Client to Server Messages

**Registration:**

```json
{
  "type": "register",
  "client_id": "workstation-01",
  "hostname": "WIN-DESKTOP"
}
```

**Confirmation:**

```json
{
  "type": "confirmation",
  "confirmation": {
    "alert_id": "123e4567-e89b-12d3-a456-426614174000",
    "client_id": "workstation-01",
    "confirmed_at": "2024-01-15T10:30:00Z",
    "hostname": "WIN-DESKTOP",
    "username": "jdoe"
  }
}
```

**Heartbeat:**

```json
{
  "type": "heartbeat"
}
```

### Server to Client Messages

**Alert:**

```json
{
  "type": "alert",
  "alert": {
    "id": "123e4567-e89b-12d3-a456-426614174000",
    "title": "System Alert",
    "message": "Critical system event detected",
    "level": "critical",
    "requires_confirmation": true,
    "sound_file": "alarm_critical.wav",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

## Running as a Service

To run as a Windows service, use tools like [NSSM](https://nssm.cc/) or [WinSW](https://github.com/winsw/winsw):

### Using NSSM

```powershell
# Download NSSM and install the service
nssm install NotificationAgent "C:\path\to\notification-agent.exe"

# Set environment variables
nssm set NotificationAgent AppEnvironmentExtra SERVER_URL=ws://server:8080/ws

# Start the service
nssm start NotificationAgent
```

## Development

### Building

```bash
cargo build
```

### Running with debug logs

```bash
$env:RUST_LOG = "debug"
cargo run
```

### Testing

```bash
cargo test
```

## Example Server

See `examples/test_server.rs` for a simple WebSocket server implementation that can send test alerts.

Run the test server:

```bash
cargo run --example test_server
```

Then in another terminal:

```bash
cargo run
```

## Security Considerations

- Use `wss://` (WebSocket Secure) for production deployments
- Implement authentication on the server side
- Validate all incoming messages
- Consider implementing client certificates for mutual TLS
- Store client credentials securely (Windows Credential Manager)

## Logging

Logs are written to stdout. Control log level with the `RUST_LOG` environment variable:

```powershell
$env:RUST_LOG = "info"  # Options: error, warn, info, debug, trace
```

## Troubleshooting

### Notifications not appearing

- Ensure Windows notifications are enabled in Settings
- Check that Focus Assist is not blocking notifications
- Verify the application ID in Windows notification settings

### No sound playing

- Check that WAV files exist in the sounds directory
- Verify audio device is working
- Check Windows volume mixer

### Connection issues

- Verify server URL is correct and reachable
- Check firewall settings
- Review logs for connection errors

## License

[Your License Here]

## Contributing

Contributions welcome! Please submit pull requests or open issues for bugs and feature requests.
