#!/usr/bin/env bash

clear

echo -e "This script checks if the server is handling incorrect HTTP methods correctly"
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [localhost]: " hostname
hostname=${hostname:-localhost}

read -r -p "Port [8080]: " port
port=${port:-8080}

echo -e "\nEnter a path that does not have a DELETE method implemented (e.g., /hello) or press enter to use the default value"
echo -e "NOTE: use a leading slash for the path (e.g., /hello) or it will not work."
echo -e "NOTE: entering only a slash will test against the root path\n"
read -r -p "Path [hello]: " path
echo -e "\n==================================\n"
path=${path:-/hello}

curl -i -X DELETE "http://${hostname}:${port}${path}"
