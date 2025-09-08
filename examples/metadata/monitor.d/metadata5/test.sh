#!/bin/bash
set -xeuf -o pipefail
echo '@@STYLUS@@ status.description="Custom (warning)"'
echo '@@STYLUS@@ status.status="orange"'
echo '@@STYLUS@@ status.metadata.key="value5"'
