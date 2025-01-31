#!/usr/bin/env bash

if [[ $UID != 0 ]]; then
    echo "Please run this script with sudo:"
    echo "sudo $0 $*"
    exit 1
fi

set -x

# Copy udev rule into rules directory
sudo cp ./udev-rule/99-luxafor-ui.rules /etc/udev/rules.d
# Reload udev
sudo udevadm control -R

