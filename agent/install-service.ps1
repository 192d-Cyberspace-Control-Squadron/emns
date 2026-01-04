# Install Notification Agent as Windows Service
# Requires NSSM (https://nssm.cc/)

param(
    [string]$ServerUrl = "ws://localhost:8080/ws",
    [string]$ClientId = $env:COMPUTERNAME,
    [string]$InstallPath = "C:\NotificationAgent",
    [string]$SoundsDir = "C:\NotificationAgent\sounds",
    [switch]$Uninstall
)

$ServiceName = "NotificationAgent"
$ExePath = Join-Path $InstallPath "notification-agent.exe"

function Test-Administrator {
    $user = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($user)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

if (-not (Test-Administrator)) {
    Write-Error "This script must be run as Administrator"
    exit 1
}

# Check if NSSM is installed
$nssm = Get-Command nssm -ErrorAction SilentlyContinue
if (-not $nssm) {
    Write-Error "NSSM is not installed. Please download from https://nssm.cc/"
    Write-Host "After installing NSSM, add it to your PATH or place nssm.exe in the script directory"
    exit 1
}

if ($Uninstall) {
    Write-Host "Uninstalling $ServiceName service..."
    
    # Stop the service if running
    $service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($service) {
        if ($service.Status -eq 'Running') {
            Write-Host "Stopping service..."
            nssm stop $ServiceName
            Start-Sleep -Seconds 2
        }
        
        Write-Host "Removing service..."
        nssm remove $ServiceName confirm
    } else {
        Write-Host "Service not found"
    }
    
    Write-Host "Uninstall complete"
    exit 0
}

# Create installation directory
if (-not (Test-Path $InstallPath)) {
    Write-Host "Creating installation directory: $InstallPath"
    New-Item -ItemType Directory -Path $InstallPath -Force | Out-Null
}

# Create sounds directory
if (-not (Test-Path $SoundsDir)) {
    Write-Host "Creating sounds directory: $SoundsDir"
    New-Item -ItemType Directory -Path $SoundsDir -Force | Out-Null
}

# Copy executable
$sourcePath = ".\target\release\notification-agent.exe"
if (-not (Test-Path $sourcePath)) {
    Write-Error "Executable not found at: $sourcePath"
    Write-Host "Please build the project first: cargo build --release"
    exit 1
}

Write-Host "Copying executable to: $ExePath"
Copy-Item $sourcePath $ExePath -Force

# Copy sound files if they exist
if (Test-Path ".\sounds\*.wav") {
    Write-Host "Copying sound files to: $SoundsDir"
    Copy-Item ".\sounds\*.wav" $SoundsDir -Force
}

# Check if service already exists
$existingService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
if ($existingService) {
    Write-Host "Service already exists. Stopping and removing..."
    nssm stop $ServiceName
    Start-Sleep -Seconds 2
    nssm remove $ServiceName confirm
}

# Install service
Write-Host "Installing service..."
nssm install $ServiceName $ExePath

# Configure service
Write-Host "Configuring service..."
nssm set $ServiceName AppDirectory $InstallPath
nssm set $ServiceName AppEnvironmentExtra "SERVER_URL=$ServerUrl" "CLIENT_ID=$ClientId" "SOUNDS_DIR=$SoundsDir"
nssm set $ServiceName DisplayName "Notification Agent"
nssm set $ServiceName Description "Receives and displays alerts from notification server"
nssm set $ServiceName Start SERVICE_AUTO_START
nssm set $ServiceName AppStdout "$InstallPath\logs\stdout.log"
nssm set $ServiceName AppStderr "$InstallPath\logs\stderr.log"
nssm set $ServiceName AppRotateFiles 1
nssm set $ServiceName AppRotateBytes 1048576

# Create logs directory
$logsDir = Join-Path $InstallPath "logs"
if (-not (Test-Path $logsDir)) {
    New-Item -ItemType Directory -Path $logsDir -Force | Out-Null
}

# Start service
Write-Host "Starting service..."
nssm start $ServiceName

# Check status
Start-Sleep -Seconds 2
$service = Get-Service -Name $ServiceName
Write-Host ""
Write-Host "Service Status: $($service.Status)" -ForegroundColor $(if ($service.Status -eq 'Running') { 'Green' } else { 'Red' })
Write-Host ""
Write-Host "Installation complete!"
Write-Host "  Install Path: $InstallPath"
Write-Host "  Server URL: $ServerUrl"
Write-Host "  Client ID: $ClientId"
Write-Host "  Sounds Dir: $SoundsDir"
Write-Host ""
Write-Host "Manage the service using:"
Write-Host "  Start:   nssm start $ServiceName"
Write-Host "  Stop:    nssm stop $ServiceName"
Write-Host "  Restart: nssm restart $ServiceName"
Write-Host "  Status:  Get-Service $ServiceName"
Write-Host "  Logs:    Get-Content $InstallPath\logs\stdout.log -Tail 50"
Write-Host ""
Write-Host "To uninstall: .\install-service.ps1 -Uninstall"
