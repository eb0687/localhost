#!/usr/bin/env bash

clear

echo -e "This script checks if the server rejects malformed URL encodings correctly."
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [localhost]: " hostname
hostname=${hostname:-localhost}

read -r -p "Port [8080]: " port
port=${port:-8080}

echo -e "\nEnter a malformed path on the server (e.g., /upload/%ZZ) or press enter to use the default value."
echo -e "NOTE: this test intentionally uses invalid percent-encoding.\n"
read -r -p "Malformed path [/upload/%ZZ]: " path
path=${path:-/upload/%ZZ}

echo -e "\n==================================\n"

curl -i "http://${hostname}:${port}${path}"
