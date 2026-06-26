#!/bin/bash
body="$(cat)"

printf "Content-Type: text/plain\r\n"
printf "\r\n"
printf "Body was: %s\n" "$body"
printf "CONTENT_LENGTH=%s\n" "$CONTENT_LENGTH"
printf "CONTENT_TYPE=%s\n" "$CONTENT_TYPE"
