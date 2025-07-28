#!/bin/bash
# Fail 50% of the time, timeout 25% of the time by sleeping for 10s, and succeed 25% of the time
if [ $((RANDOM % 2)) -eq 0 ]; then
    echo "Success"
    exit 0
elif [ $((RANDOM % 4)) -eq 0 ]; then
    echo "Timeout"
    sleep 10
    exit 0
else
    echo "Error"
    exit 1
fi
