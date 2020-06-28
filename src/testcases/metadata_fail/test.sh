#!/bin/bash
set -xeuf -o pipefail
echo '@@STYLUS@@ status.description="Custom (yellow)"'
echo '@@STYLUS@@ status.status="yellow"'
echo '@@STYLUS@@ status.metadata.key="value1"'
exit 1