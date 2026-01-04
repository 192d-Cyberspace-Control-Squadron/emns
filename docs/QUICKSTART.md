# Quick Start Guide

## Testing the Application (5 minutes)

### Step 1: Build the Project

```powershell
cargo build --release
```

### Step 2: Start the Test Server

In one terminal:

```powershell
cargo run --example test_server
```

This starts a WebSocket server on `localhost:8080` that will send test alerts.

### Step 3: Run the Agent

In another terminal:

```powershell
.\run.ps1
```

Or manually:

```powershell
$env:SERVER_URL = "ws://localhost:8080/ws"
.\target\release\notification-agent.exe
```

### Step 4: Observe the Test Alerts

The test server will send 4 alerts over about 40 seconds:

1. **Info Alert** - Blue notification, no confirmation required
2. **Warning Alert** - Yellow notification, requires confirmation
3. **Critical Alert** - Red notification, requires confirmation
4. **Emergency Alert** - Urgent notification, requires confirmation

You should see:

- Windows toast notifications appearing
- System beeps (or audio if you've added WAV files)
- Log messages in both terminals

## Adding Sound Files

1. Download or create WAV files for different alert levels
2. Place them in the `sounds` directory:
   - `sounds/alarm_critical.wav`
   - `sounds/alarm_warning.wav`
   - `sounds/notification.wav`
3. Restart the agent

## Production Deployment

### Option 1: Run as Foreground Process

```powershell
# Set production server URL
$env:SERVER_URL = "ws://alerts.yourdomain.com:8080/ws"
$env:CLIENT_ID = "workstation-$(hostname)"

# Run
.\target\release\notification-agent.exe
```

### Option 2: Install as Windows Service

1. Download NSSM: <https://nssm.cc/>
2. Run the installation script:

```powershell
.\install-service.ps1 -ServerUrl "ws://your-server:8080/ws" -ClientId "workstation-001"
```

1. Check service status:

```powershell
Get-Service NotificationAgent
```

1. View logs:

```powershell
Get-Content C:\NotificationAgent\logs\stdout.log -Tail 50 -Wait
```

## Connecting to Your Own Server

The agent expects a WebSocket server that implements the protocol documented in README.md.

Minimal server requirements:

1. Accept WebSocket connections
2. Receive registration messages
3. Send alert messages in the correct JSON format
4. Receive confirmation messages

See `examples/test_server.rs` for a complete example.

## Environment Variables

Create a `.env` file or set environment variables:

```
SERVER_URL=ws://alerts.example.com:8080/ws
CLIENT_ID=my-workstation
SOUNDS_DIR=C:\AlertSounds
RUST_LOG=info
```

## Troubleshooting

### "No notifications appearing"

- Check Windows notification settings
- Ensure Focus Assist is off
- Run as administrator if needed

### "Connection refused"

- Verify server is running
- Check firewall settings
- Confirm server URL is correct

### "No sound playing"

- Verify WAV files exist in sounds directory
- Check Windows volume mixer
- Test with system beep fallback

## Next Steps

1. **Customize Alerts**: Modify the test server to send different alert types
2. **Add Authentication**: Implement token-based auth in your production server
3. **TLS/SSL**: Use `wss://` for encrypted connections
4. **Monitoring**: Set up logging aggregation for the service
5. **Auto-start**: Configure the service to start on boot

## Support

For issues or questions:

- Check the full README.md for detailed documentation
- Review logs for error messages
- Test with the example server first
