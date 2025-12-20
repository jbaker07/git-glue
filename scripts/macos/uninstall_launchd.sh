#!/bin/bash

# uninstall_launchd.sh: Remove Model A from system

set -e

echo "Model A LaunchD Uninstallation"
echo "==============================="

if [[ $EUID -ne 0 ]]; then
    echo "ERROR: This script must be run as root (use sudo)"
    exit 1
fi

# Stop and unload services
echo "Stopping services..."
launchctl unload /Library/LaunchDaemons/com.modela.capture.plist 2>/dev/null || true
launchctl unload /Library/LaunchAgents/com.modela.locald.plist 2>/dev/null || true
launchctl unload /Library/LaunchAgents/com.modela.ui_server.plist 2>/dev/null || true

# Remove plist files
rm -f /Library/LaunchDaemons/com.modela.capture.plist
rm -f /Library/LaunchAgents/com.modela.locald.plist
rm -f /Library/LaunchAgents/com.modela.ui_server.plist

# Remove installation directory (optional)
echo "Remove /opt/modela? (y/n)"
read -r response
if [[ "$response" =~ ^[Yy]$ ]]; then
    rm -rf /opt/modela
    echo "✓ /opt/modela removed"
fi

echo "✅ Uninstallation complete"
