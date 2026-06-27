#!/usr/bin/env bash

clear

echo -e "This script checks if the server handles redirects correctly."
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [localhost]: " hostname
hostname=${hostname:-localhost}

read -r -p "Port [8080]: " port
port=${port:-8080}

echo -e "\nEnter a redirect path on the server (e.g., /redirect) or press enter to use default value"
echo -e "NOTE: use a leading slash for the path (e.g., /redirect) or it will not work.\n"
read -r -p "Redirect path [/redirect]: " path
path=${path:-/redirect}

echo -e "\n==================================\n"

curl -i http://localhost:8080/redirect
echo -e "\n==================================\n"
curl -i -L "http://${hostname}:${port}${path}"
