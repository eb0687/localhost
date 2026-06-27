#!/usr/bin/env bash

clear

echo -e "This script checks if the server handles hostname-based routing across multiple ports correctly."
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [site2.local]: " hostname
hostname=${hostname:-site2.local}

echo -e "\nEnter the ports to test, separated by commas (e.g., 8080,8021) or press enter to use default values"
read -r -p "Ports, comma-separated [8080,8021]: " ports
ports=${ports:-8080,8021}

read -r -p "Resolve IP [127.0.0.1]: " resolve_ip
resolve_ip=${resolve_ip:-127.0.0.1}

echo -e "\nEnter a path to request from this virtual host (e.g., /hello) or press enter to use default value"
echo -e "NOTE: use a leading slash for the path (e.g., /hello) or it will not work.\n"
read -r -p "Path [/hello]: " path
path=${path:-/hello}

echo -e "\n==================================\n"

IFS=',' read -ra port_list <<< "$ports"

for port in "${port_list[@]}"; do
    port="${port//[[:space:]]/}"

    echo -e "Testing ${hostname}:${port}${path}\n"

    curl --resolve "${hostname}:${port}:${resolve_ip}" -i "http://${hostname}:${port}${path}"

    echo -e "\n==================================\n"
done
