#!/bin/bash

printf "Content-Type: text/plain\r\n"
printf "\r\n"
printf "Hello world from CGI\n"
printf "REQUEST_METHOD=%s\n" "$REQUEST_METHOD"
printf "PATH_INFO=%s\n" "$PATH_INFO"
printf "QUERY_STRING=%s\n" "$QUERY_STRING"
