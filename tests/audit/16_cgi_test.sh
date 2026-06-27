#!/usr/bin/env bash

clear

echo -e "This script checks if the CGI echo script handles POST data correctly."
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [localhost]: " hostname
hostname=${hostname:-localhost}

read -r -p "Port [8080]: " port
port=${port:-8080}

echo -e "\nEnter the CGI path on the server or press enter to use default value"
echo -e "NOTE: use a leading slash for the path (e.g., /cgi/echo.sh) or it will not work.\n"
read -r -p "CGI path [/cgi/echo.sh]: " path
path=${path:-/cgi/echo.sh}

echo -e "\nEnter the body to send to the CGI script or press enter to use default value\n"
read -r -p "Body: " body
body=${body:-hello from cgi}

echo -e "\nEnter Content-Type or press enter to use default value\n"
read -r -p "Content-Type [text/plain]: " content_type
content_type=${content_type:-text/plain}

echo -e "\n==================================\n"

curl -i -X POST "http://${hostname}:${port}${path}" \
    -H "Content-Type: ${content_type}" \
    --data "$body"
