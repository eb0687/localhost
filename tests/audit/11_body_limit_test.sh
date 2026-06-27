#!/usr/bin/env bash

clear

echo -e "This script checks if the server enforces the request body size limit."
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [small.local]: " hostname
hostname=${hostname:-small.local}

read -r -p "Port [8080]: " port
port=${port:-8080}

read -r -p "Resolve IP [127.0.0.1]: " resolve_ip
resolve_ip=${resolve_ip:-127.0.0.1}

echo -e "\nEnter an upload path on the server (e.g., /upload/file.txt). or press enter to use default value"
echo -e "NOTE: use a leading slash for the path (e.g., /upload/file.txt) or it will not work.\n"
read -r -p "Upload path [/upload/file.txt]: " path
path=${path:-/upload/file.txt}

echo -e "\nEnter a request body to send (e.g., 'I am big data!') or press enter to use default value"
read -r -p "Request body: " body
body=${body:-too large body}

echo -e "\n==================================\n"

curl --resolve "${hostname}:${port}:${resolve_ip}" \
    -i -X POST "http://${hostname}:${port}${path}" \
    --data "$body"
