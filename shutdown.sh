#!/usr/bin/bash

# Name of the tmux session
SESSION="mcserver"

# Check if tmux session exists
if tmux has-session -t $SESSION 2>/dev/null; then
    echo "Sending 'stop' to Minecraft server..."
    tmux send-keys -t $SESSION "stop" C-m
    sleep 10  # Wait for server to shut down properly (adjust as needed)
else
    echo "No tmux session '$SESSION' found. Nothing to shut down."
fi

