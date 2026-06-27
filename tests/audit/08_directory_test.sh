#!/usr/bin/env bash

clear

echo -e "This script checks if the server handles access to the upload directory correctly."
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [localhost]: " hostname
hostname=${hostname:-localhost}

read -r -p "Port [8080]: " port
port=${port:-8080}

echo -e "\nEnter the upload directory path on the server (e.g., /upload) or press enter to use default value"
echo -e "NOTE: use a leading slash for the path (e.g., /upload) or it will not work.\n"
read -r -p "Upload directory path [/upload]: " path
path=${path:-/upload}

echo -e "\n==================================\n"

curl -i "http://${hostname}:${port}${path}"
