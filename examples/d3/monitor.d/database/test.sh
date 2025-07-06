#!/bin/bash
set -xeuf -o pipefail

# Simulate a database check
# In a real scenario, this would check if a database is responding
# For demo purposes, we'll simulate different states

# Simulate a database that's mostly stable but occasionally has issues
if [ $((RANDOM % 15)) -lt 13 ]; then
    # 87% chance of being up
    echo "Database is responding normally"
    exit 0
elif [ $((RANDOM % 3)) -eq 0 ]; then
    # 8% chance of timeout/slow response
    echo "Database is slow to respond"
    sleep 20  # This will cause a timeout
    exit 0
else
    # 5% chance of being down
    echo "Database connection failed"
    exit 1
fi 