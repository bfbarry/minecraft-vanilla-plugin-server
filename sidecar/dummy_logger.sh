#!/bin/bash

LOG_FILE="/Users/brianbarry/Desktop/computing/minecraft-vanilla-plugin-server/logs/dummy_log.log"
counter=1

while true; do
    timestamp=$(date +"%H:%M:%S")
    echo "[$timestamp] [Server thread/INFO]: Text here $counter" >> "$LOG_FILE"
    ((counter++))
    sleep 1
done