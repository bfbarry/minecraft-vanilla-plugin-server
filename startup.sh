#!/bin/bash

SESSION="mcserver"
USER_HOME="/home/youruser"

# ensure tmux is available
command -v tmux >/dev/null 2>&1 || exit 1

# start tmux session if not already running
sudo -u barry_brian_f tmux has-session -t $SESSION 2>/dev/null

# NOTE: don't forget gamerule sendCommandFeedback false
# TODO: if no sidecar binary, compile first
if [ $? != 0 ]; then
#    sudo -u barry_brian_f ./home/barry_brian_f/server/sidecar/command_listener & 
    sudo -u barry_brian_f tmux new-session -d -s $SESSION "java -Xmx2G -Xms1024M -jar /home/barry_brian_f/server/server.jar nogui >> /dev/null"
fi
