#!/bin/bash

# Ensure the script is run with sudo (required for setting ptrace_scope and capsh)
if [[ $EUID -ne 0 ]]; then
  echo "Please run this script with sudo:"
  echo "  sudo $0"
  exit 1
fi

# Step 1: List Proton games
echo "Available Proton App IDs:"
sudo -u $SUDO_USER protontricks -l

# Step 2: Prompt for App ID
read -p "Enter the App ID to inspect with Process Hacker: " APPID

# Optional: Validate App ID is a number
if ! [[ "$APPID" =~ ^[0-9]+$ ]]; then
  echo "Invalid App ID: $APPID"
  exit 1
fi

# Step 3: Set ptrace_scope to 0
echo "[*] Setting ptrace_scope to 0"
echo 0 > /proc/sys/kernel/yama/ptrace_scope

# Step 4: Launch Process Hacker in that Proton prefix
echo "[*] Launching Process Hacker for AppID $APPID"

# Drop capsh and just use sudo -u to avoid "Unable to set group list for user" error
sudo -u $SUDO_USER -E env HOME=/home/$SUDO_USER \
  protontricks-launch --no-bwrap --appid $APPID /home/$SUDO_USER/Downloads/processhacker/x64/ProcessHacker.exe

# Step 5: Restore ptrace_scope to 2
echo "[*] Restoring ptrace_scope to 2"
echo 2 > /proc/sys/kernel/yama/ptrace_scope

echo "[✓] Done."
