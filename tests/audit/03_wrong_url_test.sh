#!/usr/bin/env bash

clear

echo -e "This script checks if the server is handling a request to a non-existent URL correctly."
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [localhost]: " hostname
hostname=${hostname:-localhost}

read -r -p "Port [8080]: " port
port=${port:-8080}

echo -e "\nEnter a path that does not exist on the server (e.g., something) or press enter to use the default value\n"
read -r -p "Missing path [/does-not-exist]: " path
echo -e "\n==================================\n"
path=${path:-does-not-exist}

curl -i "http://${hostname}:${port}/${path}"
