#!/bin/bash
set -xeuf -o pipefail

# Simulate a cache check
# In a real scenario, this would check if a cache service is responding
# For demo purposes, we'll simulate different states

# Simulate a cache that's generally reliable but can have issues
if [ $((RANDOM % 12)) -lt 10 ]; then
    # 83% chance of being up
    echo "Cache is responding normally"
    exit 0
elif [ $((RANDOM % 2)) -eq 0 ]; then
    # 8% chance of timeout/slow response
    echo "Cache is slow to respond"
    sleep 8  # This will cause a timeout
    exit 0
else
    # 8% chance of being down
    echo "Cache service unavailable"
    exit 1
fi 