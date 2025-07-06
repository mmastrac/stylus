#!/bin/bash
set -xeuf -o pipefail

# Simulate a web server check
# In a real scenario, this would check if a web server is responding
# For demo purposes, we'll simulate different states

echo "@@STYLUS@@ status.metadata.rps=\"RPS: $(($RANDOM % 1000))\""

# Simulate a web server that's mostly up but occasionally has issues
if [ $((RANDOM % 10)) -lt 8 ]; then
    # 80% chance of being up
    echo "Web server is responding normally"
    exit 0
elif [ $((RANDOM % 2)) -eq 0 ]; then
    # 10% chance of timeout/slow response
    echo "Web server is slow to respond"
    sleep 15  # This will cause a timeout
    exit 0
else
    # 10% chance of being down
    echo "Web server is not responding"
    exit 1
fi 
