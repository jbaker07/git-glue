# run_stack_windows.ps1 - Start EDR stack on Windows
# Requires: cargo, Administrator access for capture
#
# Usage:
#   .\scripts\run_stack_windows.ps1              # Build and run
#   .\scripts\run_stack_windows.ps1 -NoBuild     # Run pre-built binaries

param(
    [switch]$NoBuild,
    [switch]$Help
)

if ($Help) {
    Write-Host "Usage: .\run_stack_windows.ps1 [-NoBuild]"
    Write-Host "  -NoBuild  Skip cargo build, use pre-built binaries"
    exit 0
}

$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent $ScriptDir
Set-Location $ProjectRoot

# Configuration
if (-not $env:EDR_TELEMETRY_ROOT) {
    $env:EDR_TELEMETRY_ROOT = "C:\ProgramData\edr"
}
$TelemetryRoot = $env:EDR_TELEMETRY_ROOT
$WaitSeconds = 60

Write-Host "=========================================="
Write-Host " EDR Stack - Windows"
Write-Host "=========================================="
Write-Host "EDR_TELEMETRY_ROOT: $TelemetryRoot"
Write-Host ""

# Create directories
$dirs = @(
    "$TelemetryRoot\segments",
    "$TelemetryRoot\incidents\default",
    "$TelemetryRoot\exports\default",
    "$TelemetryRoot\metrics"
)
foreach ($dir in $dirs) {
    if (-not (Test-Path $dir)) {
        New-Item -ItemType Directory -Path $dir -Force | Out-Null
    }
}

# Create empty index.json if not exists
$indexPath = "$TelemetryRoot\segments\index.json"
if (-not (Test-Path $indexPath)) {
    '{"segments":[]}' | Out-File -FilePath $indexPath -Encoding utf8
}

# Build if requested
if (-not $NoBuild) {
    Write-Host "[1/5] Building release binaries..."
    cargo build --release -p agent-windows -p edr-locald -p edr-server
    if ($LASTEXITCODE -ne 0) {
        Write-Host "ERROR: Build failed"
        exit 1
    }
}

# Binary paths
$CaptureBin = "$ProjectRoot\target\release\capture_windows_rotating.exe"
$LocaldBin = "$ProjectRoot\target\release\edr-locald.exe"
$ServerBin = "$ProjectRoot\target\release\edr-server.exe"

# Verify binaries exist
$binaries = @($CaptureBin, $LocaldBin, $ServerBin)
foreach ($bin in $binaries) {
    if (-not (Test-Path $bin)) {
        Write-Host "ERROR: Binary not found: $bin"
        Write-Host "Run without -NoBuild to compile"
        exit 1
    }
}

# Process tracking for cleanup
$Processes = @()

function Cleanup {
    Write-Host ""
    Write-Host "[SHUTDOWN] Stopping all processes..."
    foreach ($proc in $script:Processes) {
        if (-not $proc.HasExited) {
            Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        }
    }
    Write-Host "[SHUTDOWN] Complete"
}

# Register cleanup on exit
Register-EngineEvent PowerShell.Exiting -Action { Cleanup }

try {
    Write-Host "[2/5] Starting capture_windows_rotating..."
    $captureProc = Start-Process -FilePath $CaptureBin -NoNewWindow -PassThru -RedirectStandardOutput "$TelemetryRoot\capture.log" -RedirectStandardError "$TelemetryRoot\capture_err.log"
    $Processes += $captureProc
    Write-Host "  PID: $($captureProc.Id)"

    Start-Sleep -Seconds 2

    Write-Host "[3/5] Starting edr-locald..."
    $localdProc = Start-Process -FilePath $LocaldBin -NoNewWindow -PassThru -RedirectStandardOutput "$TelemetryRoot\locald.log" -RedirectStandardError "$TelemetryRoot\locald_err.log"
    $Processes += $localdProc
    Write-Host "  PID: $($localdProc.Id)"

    Start-Sleep -Seconds 1

    Write-Host "[4/5] Starting edr-server..."
    $serverProc = Start-Process -FilePath $ServerBin -NoNewWindow -PassThru -RedirectStandardOutput "$TelemetryRoot\server.log" -RedirectStandardError "$TelemetryRoot\server_err.log"
    $Processes += $serverProc
    Write-Host "  PID: $($serverProc.Id)"

    Write-Host ""
    Write-Host "=========================================="
    Write-Host " Stack Running"
    Write-Host "=========================================="
    Write-Host "UI URL:        http://localhost:3000"
    Write-Host "API Health:    http://localhost:3000/api/health"
    Write-Host "Incidents:     http://localhost:3000/api/incidents"
    Write-Host ""
    Write-Host "Logs:"
    Write-Host "  Capture:  Get-Content $TelemetryRoot\capture.log -Wait"
    Write-Host "  Locald:   Get-Content $TelemetryRoot\locald.log -Wait"
    Write-Host "  Server:   Get-Content $TelemetryRoot\server.log -Wait"
    Write-Host ""

    Write-Host "[5/5] Waiting $WaitSeconds seconds for events to accumulate..."
    Start-Sleep -Seconds $WaitSeconds

    # Run proof_run
    Write-Host ""
    Write-Host "=========================================="
    Write-Host " Running proof_run"
    Write-Host "=========================================="
    $ProofRunBin = "$ProjectRoot\target\release\proof_run.exe"
    if (Test-Path $ProofRunBin) {
        & $ProofRunBin
        Write-Host ""
        Write-Host "Proof artifacts:"
        Get-ChildItem -Path $TelemetryRoot -Recurse -Include "proof_run*.json","incidents*.jsonl" | Select-Object -First 10 | ForEach-Object { Write-Host $_.FullName }
    } else {
        Write-Host "[WARN] proof_run binary not found. Build with: cargo build --release -p edr-locald"
    }

    Write-Host ""
    Write-Host "=========================================="
    Write-Host " Summary"
    Write-Host "=========================================="
    $segmentCount = (Get-ChildItem -Path "$TelemetryRoot\segments" -Filter "*.jsonl" -ErrorAction SilentlyContinue | Measure-Object).Count
    $incidentCount = (Get-ChildItem -Path "$TelemetryRoot\incidents\default" -Filter "*.json" -ErrorAction SilentlyContinue | Measure-Object).Count
    Write-Host "Segments written: $segmentCount"
    Write-Host "Incidents:        $incidentCount"
    Write-Host ""
    Write-Host "Press Ctrl-C to stop the stack..."
    Write-Host ""

    # Wait for interrupt
    while ($true) {
        Start-Sleep -Seconds 1
        # Check if any process has exited
        foreach ($proc in $Processes) {
            if ($proc.HasExited) {
                Write-Host "[WARN] Process $($proc.Id) exited with code $($proc.ExitCode)"
            }
        }
    }
} finally {
    Cleanup
}
