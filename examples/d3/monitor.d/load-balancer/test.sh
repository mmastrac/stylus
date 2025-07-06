#!/bin/bash
set -xeuf -o pipefail

# Simulate a load balancer check
# In a real scenario, this would check if a load balancer is healthy
# For demo purposes, we'll simulate different states

# Simulate a load balancer that's very reliable
if [ $((RANDOM % 20)) -lt 19 ]; then
    # 95% chance of being up
    echo "Load balancer is healthy"
    exit 0
elif [ $((RANDOM % 2)) -eq 0 ]; then
    # 2.5% chance of timeout/slow response
    echo "Load balancer is slow to respond"
    sleep 12  # This will cause a timeout
    exit 0
else
    # 2.5% chance of being down
    echo "Load balancer health check failed"
    exit 1
fi 