#!/bin/bash

# Enables Hidepid changes mac
sudo clear
echo "Enabling Hidepid"
sudo mount -o remount,rw,hidepid=2 /proc
echo "Done..."
sleep 1

if ! sysctl kernel.yama.ptrace_scope | grep -q '2'; then
    sudo sysctl -w kernel.yama.ptrace_scope=2
    echo "ptrace_scope was not 2, now set to 2"
else
    echo "ptrace_scope is already 2"
fi
sleep 1
echo "Changing Macaddress of all network interfaces"
interfaces=$(ip a | awk -F: '/^[0-9]+:/{print $2}' | tr -d ' ')

# Loop through each interface
for iface in $interfaces; do
    sudo macchanger --random $iface
done
