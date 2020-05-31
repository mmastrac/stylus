#!/bin/bash
set -xeuf -o pipefail
echo '@@STYLUS@@ status.description="Custom (red)"'
echo '@@STYLUS@@ status.status="red"'
echo '@@STYLUS@@ status.metadata.key="value2"'
