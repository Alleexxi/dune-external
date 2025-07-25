#!/bin/bash

if [[ $EUID -ne 0 ]]; then
  echo "Please run this script with sudo:"
  echo "  sudo $0"
  exit 1
fi

echo "Available Proton App IDs:"
sudo -u $SUDO_USER protontricks -l

read -p "Enter the App ID to inspect with Process Hacker: " APPID

if ! [[ "$APPID" =~ ^[0-9]+$ ]]; then
  echo "Invalid App ID: $APPID"
  exit 1
fi

echo "[*] Setting ptrace_scope to 0"
echo 0 > /proc/sys/kernel/yama/ptrace_scope

echo "[*] Launching Process Hacker for AppID $APPID"

sudo -u $SUDO_USER -E env HOME=/home/$SUDO_USER \
  protontricks-launch --no-bwrap --appid $APPID /home/$SUDO_USER/Downloads/processhacker/x64/ProcessHacker.exe

echo "[*] Restoring ptrace_scope to 2"
echo 2 > /proc/sys/kernel/yama/ptrace_scope

echo "[✓] Done."
