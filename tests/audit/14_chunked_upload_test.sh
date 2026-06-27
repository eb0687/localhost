#!/usr/bin/env bash

clear

echo -e "This script checks if the server handles chunked uploads correctly."
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [localhost]: " hostname
hostname=${hostname:-localhost}

read -r -p "Port [8080]: " port
port=${port:-8080}

echo -e "\nEnter an upload path on the server (e.g., /upload/chunked.txt) or press enter to use default value"
echo -e "NOTE: use a leading slash for the path (e.g., /upload/chunked.txt) or it will not work.\n"
read -r -p "Upload path [/upload/chunked.txt]: " path
path=${path:-/upload/chunked.txt}

echo -e "\nEnter the body of the upload or press enter to use default value\n"
read -r -p "Body: " body
body=${body:-hello chunked}

echo -e "\n==================================\n"

curl -i -X POST "http://${hostname}:${port}${path}" \
    -H "Transfer-Encoding: chunked" \
    --data-binary "$body"
