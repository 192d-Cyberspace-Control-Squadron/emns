# Run Notification Agent in development mode
# This script sets up environment variables and runs the agent

param(
    [string]$ServerUrl = "ws://localhost:8080/ws",
    [string]$ClientId = $env:COMPUTERNAME,
    [string]$SoundsDir = "./sounds",
    [string]$LogLevel = "info"
)

Write-Host "Starting Notification Agent..." -ForegroundColor Green
Write-Host "  Server URL: $ServerUrl"
Write-Host "  Client ID: $ClientId"
Write-Host "  Sounds Dir: $SoundsDir"
Write-Host "  Log Level: $LogLevel"
Write-Host ""

# Set environment variables
$env:SERVER_URL = $ServerUrl
$env:CLIENT_ID = $ClientId
$env:SOUNDS_DIR = $SoundsDir
$env:RUST_LOG = $LogLevel

# Create sounds directory if it doesn't exist
if (-not (Test-Path $SoundsDir)) {
    New-Item -ItemType Directory -Path $SoundsDir -Force | Out-Null
    Write-Host "Created sounds directory: $SoundsDir" -ForegroundColor Yellow
}

# Check if executable exists
$exePath = ".\target\release\notification-agent.exe"
if (-not (Test-Path $exePath)) {
    $exePath = ".\target\debug\notification-agent.exe"
    if (-not (Test-Path $exePath)) {
        Write-Host "Building project..." -ForegroundColor Yellow
        cargo build
        $exePath = ".\target\debug\notification-agent.exe"
    }
}

Write-Host "Running: $exePath" -ForegroundColor Cyan
Write-Host ""
Write-Host "Press Ctrl+C to stop" -ForegroundColor Gray
Write-Host ""

# Run the agent
& $exePath
