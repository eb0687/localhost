#!/usr/bin/env bash

clear

echo -e "This script checks if the server handles hostname-based routing correctly."
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [site1.local]: " hostname
hostname=${hostname:-site1.local}

read -r -p "Port [8080]: " port
port=${port:-8080}

read -r -p "Resolve IP [127.0.0.1]: " resolve_ip
resolve_ip=${resolve_ip:-127.0.0.1}

echo -e "\nEnter a path to request from this virtual host (e.g., /) or press enter to use default value"
echo -e "NOTE: use a leading slash for the path (e.g., /) or it will not work.\n"
read -r -p "Path [/]: " path
path=${path:-/}

echo -e "\n==================================\n"

curl --resolve "${hostname}:${port}:${resolve_ip}" -i "http://${hostname}:${port}${path}"
