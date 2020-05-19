#!/bin/bash
set -xeuf -o pipefail
echo '@@STYLUS@@ status.description="Custom!"'
echo '@@STYLUS@@ status.status="yellow"'
echo '@@STYLUS@@ status.metadata.key="value"'
