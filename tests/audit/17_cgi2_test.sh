#!/usr/bin/env bash

clear

echo -e "This script checks if the CGI hello script handles GET and query strings correctly."
echo -e "Press Enter to use the default values.\n"

read -r -p "Hostname [localhost]: " hostname
hostname=${hostname:-localhost}

read -r -p "Port [8080]: " port
port=${port:-8080}

echo -e "\nEnter the CGI path on the server or press enter to use default value"
echo -e "NOTE: use a leading slash for the path (e.g., /cgi/hello.sh) or it will not work.\n"
read -r -p "CGI path [/cgi/hello.sh]: " path
path=${path:-/cgi/hello.sh}

echo -e "\nEnter query string without '?' or press enter to use default value\n"
read -r -p "Query [name=audit&lang=bash]: " query
query=${query:-name=audit&lang=bash}

echo -e "\n==================================\n"

curl -i "http://${hostname}:${port}${path}?${query}"
