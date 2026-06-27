#!/usr/bin/env bash

clear

echo -e "This script checks if the server is running by sending a request to the root URL."
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [localhost]: " hostname
hostname=${hostname:-localhost}

read -r -p "Port [8080]: " port
echo -e "\n==================================\n"
port=${port:-8080}

curl -i "http://${hostname}:${port}/"
