#!/bin/bash

# install_launchd.sh: Install Model A as persistent macOS LaunchDaemon + LaunchAgent
# 
# Usage:
#   sudo -E ./install_launchd.sh [--telemetry-root <path>]
#
# Requirements:
#   - Run with sudo (for daemon)
#   - Set EDR_TELEMETRY_ROOT env var or use --telemetry-root
#   - Binaries must be built: target/release/capture_macos_rotating edr_locald ui_server

set -e

TELEMETRY_ROOT="${EDR_TELEMETRY_ROOT:-./telemetry_output}"
INSTALL_PREFIX="/opt/modela"
LOG_DIR="/var/log/modela"
USER="${SUDO_USER:-root}"

# Parse CLI args
while [[ $# -gt 0 ]]; do
    case $1 in
        --telemetry-root)
            TELEMETRY_ROOT="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "Model A LaunchD Installation"
echo "=============================="
echo "TELEMETRY_ROOT: $TELEMETRY_ROOT"
echo "INSTALL_PREFIX: $INSTALL_PREFIX"
echo "USER: $USER"

# Verify running as root
if [[ $EUID -ne 0 ]]; then
    echo "ERROR: This script must be run as root (use sudo)"
    exit 1
fi

# Verify binaries exist
if [ ! -f "target/release/capture_macos_rotating" ] || \
   [ ! -f "target/release/edr_locald" ] || \
   [ ! -f "target/release/ui_server" ]; then
    echo "ERROR: Binaries not found in target/release/"
    echo "Run: cargo build --release"
    exit 1
fi

# Create installation directories
mkdir -p "$INSTALL_PREFIX/bin"
mkdir -p "$LOG_DIR"
mkdir -p "$TELEMETRY_ROOT/segments"

# Copy binaries
echo "Installing binaries..."
cp target/release/capture_macos_rotating "$INSTALL_PREFIX/bin/"
cp target/release/edr_locald "$INSTALL_PREFIX/bin/"
cp target/release/ui_server "$INSTALL_PREFIX/bin/"
chmod 755 "$INSTALL_PREFIX/bin"/*

# Copy playbooks
if [ -d "playbooks/macos" ]; then
    mkdir -p "$INSTALL_PREFIX/playbooks/macos"
    cp playbooks/macos/*.yaml "$INSTALL_PREFIX/playbooks/macos/" 2>/dev/null || true
    echo "Playbooks installed to $INSTALL_PREFIX/playbooks/macos"
fi

# Create LaunchDaemon plist for capture (runs as root, needs elevated privs for auditpipe)
cat > /Library/LaunchDaemons/com.modela.capture.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.modela.capture</string>
    <key>Program</key>
    <string>$INSTALL_PREFIX/bin/capture_macos_rotating</string>
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_PREFIX/bin/capture_macos_rotating</string>
        <string>--telemetry-root</string>
        <string>$TELEMETRY_ROOT</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$LOG_DIR/capture.out.log</string>
    <key>StandardErrorPath</key>
    <string>$LOG_DIR/capture.err.log</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>EDR_TELEMETRY_ROOT</key>
        <string>$TELEMETRY_ROOT</string>
    </dict>
</dict>
</plist>
EOF

chmod 644 /Library/LaunchDaemons/com.modela.capture.plist
echo "✓ LaunchDaemon installed: com.modela.capture"

# Create LaunchAgent plist for edr_locald (runs as user)
cat > /Library/LaunchAgents/com.modela.locald.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.modela.locald</string>
    <key>Program</key>
    <string>$INSTALL_PREFIX/bin/edr_locald</string>
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_PREFIX/bin/edr_locald</string>
        <string>--telemetry-root</string>
        <string>$TELEMETRY_ROOT</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$LOG_DIR/locald.out.log</string>
    <key>StandardErrorPath</key>
    <string>$LOG_DIR/locald.err.log</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>EDR_TELEMETRY_ROOT</key>
        <string>$TELEMETRY_ROOT</string>
    </dict>
</dict>
</plist>
EOF

chmod 644 /Library/LaunchAgents/com.modela.locald.plist
echo "✓ LaunchAgent installed: com.modela.locald"

# Create LaunchAgent plist for ui_server
cat > /Library/LaunchAgents/com.modela.ui_server.plist << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.modela.ui_server</string>
    <key>Program</key>
    <string>$INSTALL_PREFIX/bin/ui_server</string>
    <key>ProgramArguments</key>
    <array>
        <string>$INSTALL_PREFIX/bin/ui_server</string>
        <string>--telemetry-root</string>
        <string>$TELEMETRY_ROOT</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>$LOG_DIR/ui_server.out.log</string>
    <key>StandardErrorPath</key>
    <string>$LOG_DIR/ui_server.err.log</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>EDR_TELEMETRY_ROOT</key>
        <string>$TELEMETRY_ROOT</string>
    </dict>
</dict>
</plist>
EOF

chmod 644 /Library/LaunchAgents/com.modela.ui_server.plist
echo "✓ LaunchAgent installed: com.modela.ui_server"

# Start services
echo "Starting services..."
launchctl load /Library/LaunchDaemons/com.modela.capture.plist 2>/dev/null || true
launchctl load /Library/LaunchAgents/com.modela.locald.plist 2>/dev/null || true
launchctl load /Library/LaunchAgents/com.modela.ui_server.plist 2>/dev/null || true

echo ""
echo "✅ Installation complete"
echo ""
echo "Verify installation:"
echo "  launchctl list | grep modela"
echo "  tail -f $LOG_DIR/*.log"
echo "  curl http://127.0.0.1:8000/api/health"
echo ""
echo "To uninstall:"
echo "  sudo ./scripts/macos/uninstall_launchd.sh"
