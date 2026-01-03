# VM Bringup Guide

Quick guide to deploy EDR stack across macOS (native), Linux VM, and Windows VM.

## Prerequisites

### All Platforms
- Rust toolchain: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Git (to clone the repo)

### macOS (Native)
- Xcode Command Line Tools: `xcode-select --install`
- Sudo access (for BSM/audit access)
- Audit enabled: `sudo launchctl load -w /System/Library/LaunchDaemons/com.apple.auditd.plist`

### Linux VM (Ubuntu 22.04+)
```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev clang llvm
# For eBPF:
sudo apt install -y linux-headers-$(uname -r) bpftool
```

### Windows VM (Windows 10/11)
- Visual Studio Build Tools with C++ workload
- Windows SDK (for ETW)
- PowerShell 5.1+ (pre-installed)

## Quick Start

### 1. Clone Repository

```bash
git clone <repo-url> git-glue
cd git-glue
```

### 2. Build (One Command)

```bash
# macOS
cargo build --release -p agent-macos -p edr-locald -p edr-server

# Linux
cargo build --release -p agent-linux -p edr-locald -p edr-server

# Windows (PowerShell)
cargo build --release -p agent-windows -p edr-locald -p edr-server
```

### 3. Run Stack (One Command)

```bash
# macOS
./scripts/run_stack_macos.sh

# Linux
./scripts/run_stack_linux.sh

# Windows (PowerShell as Administrator)
.\scripts\run_stack_windows.ps1
```

### 4. Verify

```bash
# macOS/Linux
./scripts/smoke_stack.sh

# Windows
.\scripts\smoke_stack.ps1
```

## What Gets Created

```
$EDR_TELEMETRY_ROOT/
├── segments/
│   ├── index.json
│   └── segment_000000.jsonl
├── incidents/
│   └── default/
│       └── incident_<uuid>.json
├── exports/
│   └── default/
├── metrics/
├── capture.log
├── locald.log
└── server.log
```

## Default Telemetry Roots

| Platform | Default Path |
|----------|-------------|
| macOS    | `./telemetry_data` (repo-local) |
| Linux    | `/var/lib/edr` |
| Windows  | `C:\ProgramData\edr` |

Override with: `export EDR_TELEMETRY_ROOT=/custom/path`

## Copying Artifacts Between Host/VM

### From macOS Host to Linux VM

```bash
# On macOS (host)
scp -r target/release/{edr-server,edr-locald} user@linux-vm:/home/user/edr/

# On Linux VM
chmod +x /home/user/edr/*
export EDR_TELEMETRY_ROOT=/var/lib/edr
sudo /home/user/edr/capture_linux_rotating &
/home/user/edr/edr-locald &
/home/user/edr/edr-server &
```

### Cross-compile for Linux (from macOS)

```bash
rustup target add x86_64-unknown-linux-gnu
# Requires linker setup - easier to build on target
```

### From macOS Host to Windows VM

```powershell
# Use shared folder or SCP
# On Windows VM:
$env:EDR_TELEMETRY_ROOT = "C:\ProgramData\edr"
.\capture_windows_rotating.exe
.\edr-locald.exe
.\edr-server.exe
```

## Confirm Success

### 1. Segments Exist
```bash
ls -la $EDR_TELEMETRY_ROOT/segments/*.jsonl
# Should see segment files with size > 0
```

### 2. Incidents Created
```bash
ls -la $EDR_TELEMETRY_ROOT/incidents/default/
# May be empty initially - generate activity first
```

### 3. API Responds
```bash
curl http://localhost:3000/api/health
# {"status":"ok",...}

curl http://localhost:3000/api/capabilities
# {"sources":[...],...}
```

### 4. proof_run Artifact
```bash
./target/release/proof_run
ls $EDR_TELEMETRY_ROOT/proof_run*.json
```

## Troubleshooting

### macOS: "Failed to open /dev/auditpipe"
```bash
# Enable audit daemon
sudo launchctl load -w /System/Library/LaunchDaemons/com.apple.auditd.plist
# Verify
sudo praudit -l /dev/auditpipe
```

### Linux: "eBPF not available"
```bash
# Check kernel config
cat /boot/config-$(uname -r) | grep BPF
# Install headers
sudo apt install linux-headers-$(uname -r)
```

### Windows: "ETW initialization failed"
- Run PowerShell as Administrator
- Ensure Windows Event Log service is running

### All: "Connection refused" on port 8080
- Check if edr-server is running: `ps aux | grep edr-server`
- Check logs: `tail -f $EDR_TELEMETRY_ROOT/server.log`

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Per-Platform                          │
├─────────────────────────────────────────────────────────┤
│  capture_macos_rotating    (BSM/auditpipe)              │
│  capture_linux_rotating    (eBPF ringbuf)               │
│  capture_windows_rotating  (ETW/WEVTAPI)                │
└───────────────────────┬─────────────────────────────────┘
                        │ segments/*.jsonl
                        ▼
┌─────────────────────────────────────────────────────────┐
│                    edr-locald                            │
│  - Reads segments                                        │
│  - Slot matching (playbooks)                             │
│  - Hypothesis → Incident promotion                       │
│  - Exports JSONL                                         │
└───────────────────────┬─────────────────────────────────┘
                        │ incidents/*.json
                        ▼
┌─────────────────────────────────────────────────────────┐
│                    edr-server                            │
│  - HTTP API (8080)                                       │
│  - UI serving                                            │
│  - Integration endpoints                                 │
└─────────────────────────────────────────────────────────┘
```

## Binaries Reference

| Binary | Package | Purpose |
|--------|---------|---------|
| `capture_macos_rotating` | agent-macos | macOS BSM capture |
| `capture_linux_rotating` | agent-linux | Linux eBPF capture |
| `capture_windows_rotating` | agent-windows | Windows ETW capture |
| `edr-locald` | edr-locald | Incident compiler |
| `edr-server` | edr-server | HTTP API + UI |
| `proof_run` | edr-locald | Integration verification |
| `golden-cli` | edr-server | CLI tools |
