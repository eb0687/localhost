#!/usr/bin/env bash

clear

echo -e "This script checks if the server rejects encoded path traversal attempts correctly."
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [localhost]: " hostname
hostname=${hostname:-localhost}

read -r -p "Port [8080]: " port
port=${port:-8080}

echo -e "\nEnter an encoded path traversal attempt (e.g., /upload/%2e%2e/secret.txt) or press enter to use the default value."
echo -e "NOTE: this test intentionally uses percent-encoded '..' path traversal.\n"
read -r -p "Traversal path [/upload/%2e%2e/Cargo.toml]: " path
path=${path:-/upload/%2e%2e/secret.txt}

echo -e "\n==================================\n"

curl --path-as-is -i "http://${hostname}:${port}${path}"
