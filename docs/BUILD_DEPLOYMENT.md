# Build and Deployment Guide

## Air Force / DoD Deployment Considerations

This guide provides specific considerations for deploying in Air Force and DoD environments.

## STIG Compliance Considerations

### Hardening Checklist

- [ ] Run service with least privilege account
- [ ] Enable TLS 1.2+ only (wss://)
- [ ] Disable insecure ciphers
- [ ] Implement certificate validation
- [ ] Enable audit logging
- [ ] Set file permissions appropriately
- [ ] Use approved cryptographic modules

### Configuration for STIG Compliance

```powershell
# Run as dedicated service account (not SYSTEM)
$serviceAccount = "NT SERVICE\NotificationAgent"

# Set file permissions
icacls C:\NotificationAgent /grant "${serviceAccount}:(OI)(CI)RX"
icacls C:\NotificationAgent\logs /grant "${serviceAccount}:(OI)(CI)M"

# Configure service
sc.exe config NotificationAgent obj= $serviceAccount
```

## Building for Disconnected Networks

### Offline Build Process

1. **On Connected System:**

```powershell
# Download all dependencies
cargo fetch

# Create vendor directory
cargo vendor > .cargo/config.toml

# Package for transfer
Compress-Archive -Path notification-agent -DestinationPath notification-agent.zip
```

1. **On Disconnected System:**

```powershell
# Extract package
Expand-Archive notification-agent.zip

# Build from vendor
cd notification-agent
cargo build --release --offline
```

## Certificate-Based Authentication

### Client Certificate Setup

Add to `Cargo.toml`:

```toml
native-tls = "0.2"
tokio-native-tls = "0.3"
```

Modify `client.rs` for mTLS:

```rust
use tokio_native_tls::{native_tls, TlsConnector};

// Load client certificate
let cert = std::fs::read("client.pfx")?;
let identity = native_tls::Identity::from_pkcs12(&cert, "password")?;

let connector = TlsConnector::from(
    native_tls::TlsConnector::builder()
        .identity(identity)
        .build()?
);
```

## CAC Integration (Optional Future Enhancement)

For Common Access Card authentication:

```toml
# Additional dependencies
pcsc = "2.8"
x509-parser = "0.15"
```

## Group Policy Deployment

### MSI Package Creation

Use [WiX Toolset](https://wixtoolset.org/) to create MSI:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
  <Product Id="*" Name="Notification Agent" Version="1.0.0" 
           Manufacturer="Your Unit" Language="1033">
    <Package InstallerVersion="200" Compressed="yes" />
    
    <Directory Id="TARGETDIR" Name="SourceDir">
      <Directory Id="ProgramFilesFolder">
        <Directory Id="INSTALLFOLDER" Name="NotificationAgent">
          <Component Id="NotificationAgent" Guid="YOUR-GUID-HERE">
            <File Source="notification-agent.exe" />
            <ServiceInstall Id="ServiceInstaller"
                          Name="NotificationAgent"
                          DisplayName="Notification Agent"
                          Description="Alert Notification Service"
                          Start="auto"
                          Type="ownProcess"
                          ErrorControl="normal" />
            <ServiceControl Id="StartService"
                          Name="NotificationAgent"
                          Start="install"
                          Stop="both"
                          Remove="uninstall" />
          </Component>
        </Directory>
      </Directory>
    </Directory>
  </Product>
</Wix>
```

### GPO Deployment Steps

1. Build MSI package
2. Copy to network share
3. Create GPO:
   - Computer Configuration → Policies → Software Settings
   - Right-click Software Installation → New → Package
   - Select your MSI
   - Choose "Assigned"

4. Configure environment via registry:

```powershell
# Create registry-based config
New-Item -Path "HKLM:\SOFTWARE\NotificationAgent" -Force
New-ItemProperty -Path "HKLM:\SOFTWARE\NotificationAgent" -Name "ServerURL" -Value "wss://alerts.af.mil:8080/ws"
New-ItemProperty -Path "HKLM:\SOFTWARE\NotificationAgent" -Name "ClientID" -Value "%COMPUTERNAME%"
```

Modify code to read from registry:

```rust
use winreg::RegKey;

let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
let config_key = hklm.open_subkey("SOFTWARE\\NotificationAgent")?;
let server_url: String = config_key.get_value("ServerURL")?;
```

## NIPR/SIPR Considerations

### NIPR Deployment

- Standard deployment process
- Use `.mil` domain for server
- Enable full logging
- No special restrictions

### SIPR Deployment

- Ensure all dependencies are approved
- No external network access during build
- Use offline build process
- Implement additional encryption
- Enhanced audit logging
- No telemetry or crash reporting

## Building Different Configurations

### Debug Build (Development)

```powershell
cargo build
```

### Release Build (Production)

```powershell
cargo build --release
```

### Optimized Release Build

```powershell
# Add to Cargo.toml:
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true

cargo build --release
```

### Static Linking (Minimal Dependencies)

```powershell
# For systems without MSVC runtime
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

## Continuous Integration

### Example GitLab CI (.gitlab-ci.yml)

```yaml
stages:
  - build
  - test
  - deploy

build:
  stage: build
  script:
    - cargo build --release
  artifacts:
    paths:
      - target/release/notification-agent.exe
    expire_in: 1 week

test:
  stage: test
  script:
    - cargo test
    - cargo clippy -- -D warnings

deploy:
  stage: deploy
  script:
    - copy target\release\notification-agent.exe \\share\deployments\
  only:
    - main
```

## Deployment Automation

### PowerShell Deployment Script

```powershell
# deploy.ps1
param(
    [string[]]$Targets = @(),
    [string]$Package = ".\notification-agent.zip"
)

foreach ($target in $Targets) {
    Write-Host "Deploying to $target..."
    
    # Copy package
    Copy-Item $Package "\\$target\C$\Temp\"
    
    # Remote install
    Invoke-Command -ComputerName $target -ScriptBlock {
        # Stop service
        Stop-Service NotificationAgent -ErrorAction SilentlyContinue
        
        # Extract
        Expand-Archive C:\Temp\notification-agent.zip -Force
        
        # Install
        & C:\Temp\notification-agent\install-service.ps1
        
        # Cleanup
        Remove-Item C:\Temp\notification-agent* -Recurse -Force
    }
    
    Write-Host "Deployed to $target" -ForegroundColor Green
}
```

Usage:

```powershell
.\deploy.ps1 -Targets @("workstation-001", "workstation-002")
```

## Monitoring Integration

### Windows Event Log Integration

Add to dependencies:

```toml
eventlog = "0.2"
```

Code:

```rust
use eventlog::Logger;

let logger = Logger::new("NotificationAgent")?;
logger.info("Alert received")?;
logger.error("Connection failed")?;
```

### Syslog Integration

```toml
syslog = "6.0"
```

```rust
use syslog::{Facility, Formatter3164};

let formatter = Formatter3164 {
    facility: Facility::LOG_USER,
    hostname: None,
    process: "notification-agent".into(),
    pid: 0,
};

let logger = syslog::unix(formatter)?;
logger.err("Critical error")?;
```

## Performance Tuning

### For High-Volume Environments

```toml
[dependencies]
tokio = { version = "1.35", features = ["full", "parking_lot"] }
```

```rust
// Increase channel buffer
let (alert_tx, alert_rx) = mpsc::channel::<Alert>(1000);

// Connection pooling for confirmations
// Batch confirmations every 100ms
```

## Rollback Procedure

```powershell
# Stop service
Stop-Service NotificationAgent

# Restore previous version
Copy-Item C:\NotificationAgent\backup\notification-agent.exe C:\NotificationAgent\

# Start service
Start-Service NotificationAgent

# Verify
Get-Service NotificationAgent
```

## Health Check Endpoint (Optional)

Add simple HTTP health check:

```rust
use warp::Filter;

#[tokio::main]
async fn main() {
    // Health check endpoint
    let health = warp::path("health")
        .map(|| warp::reply::json(&json!({"status": "healthy"})));
    
    tokio::spawn(warp::serve(health).run(([127, 0, 0, 1], 9090)));
    
    // ... rest of application
}
```

## Pre-Deployment Testing

### Test Checklist

- [ ] Build completes without errors
- [ ] Unit tests pass
- [ ] Integration tests pass with test server
- [ ] Notifications display correctly
- [ ] Audio plays correctly
- [ ] Service installs successfully
- [ ] Service starts automatically
- [ ] Logs are written properly
- [ ] Confirmations are received
- [ ] Reconnection works after network interruption
- [ ] Performance acceptable under load
- [ ] No memory leaks over 24 hours

### Load Testing

```rust
// Simple load test in test_server.rs
async fn stress_test() {
    for i in 0..1000 {
        send_alert(format!("Alert {}", i)).await;
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
}
```

## Documentation Package

For deployment, include:

1. README.md
2. QUICKSTART.md
3. SERVER_GUIDE.md
4. This BUILD_DEPLOYMENT.md
5. Configuration templates
6. Service installation scripts
7. Troubleshooting guide

## Support Contacts

Document your support structure:

- Primary: Unit Comm Squadron
- Secondary: Base Comm focal point
- Escalation: MAJCOM help desk
- Emergency: [Your contact info]
