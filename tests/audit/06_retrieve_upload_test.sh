#!/usr/bin/env bash

clear

echo -e "This script checks if the server can serve an uploaded file."
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [localhost]: " hostname
hostname=${hostname:-localhost}

read -r -p "Port [8080]: " port
port=${port:-8080}

echo -e "\nEnter the uploaded file path on the server (e.g., /upload/test.txt) or press enter to use default value"
echo -e "NOTE: use a leading slash for the path (e.g., /upload/test.txt) or it will not work.\n"
read -r -p "Uploaded file path [/upload/test.txt]: " path
path=${path:-/upload/test.txt}

echo -e "\n==================================\n"

curl -i "http://${hostname}:${port}${path}"
