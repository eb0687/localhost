#!/usr/bin/env bash

clear

echo -e "This script checks if the server is serving a static file by sending a request to a specific URL."
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [localhost]: " hostname
hostname=${hostname:-localhost}

read -r -p "Port [8080]: " port
port=${port:-8080}

echo -e "\nEnter a path to a static file that exists in the server (e.g., hello)."
echo -e "If you don't know, just press Enter to use the default path /hello.\n"
read -r -p "Static path [hello]: " path
echo -e "\n==================================\n"
path=${path:-hello}

curl -i "http://${hostname}:${port}/${path}"
