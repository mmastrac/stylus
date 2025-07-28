#!/bin/bash
set -xeuf -o pipefail
echo '@@STYLUS@@ group.port-0.status.status="yellow"'
echo '@@STYLUS@@ group.port-1.status.status="green"'
echo '@@STYLUS@@ group.port-2.status.status="yellow"'
exit 1
