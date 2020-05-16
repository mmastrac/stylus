#!/bin/bash
echo Curling
curl --retry 2 --max-time 5 --connect-timeout 5 http://192.168.1.1:9000
