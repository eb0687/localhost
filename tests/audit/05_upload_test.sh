#!/usr/bin/env bash

clear

echo -e "This script checks if the server is handling file uploads correctly."
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [localhost]: " hostname
hostname=${hostname:-localhost}

read -r -p "Port [8080]: " port
port=${port:-8080}

echo -e "\nEnter an upload path on the server (e.g., /upload/test.txt) or press enter to use default value"
echo -e "NOTE: use a leading slash for the path (e.g., /upload/test.txt) or it will not work.\n"
read -r -p "Upload path [/upload/test.txt]: " path
path=${path:-/upload/test.txt}

echo -e "\nEnter the data to be uploaded. If you don't enter anything, it will default to 'hello tester'.\n"
read -r -p "Data: " body
echo -e "\n==================================\n"
body=${body:-hello tester}

curl -i -X POST "http://${hostname}:${port}${path}" --data "$body"
