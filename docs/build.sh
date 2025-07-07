#!/bin/bash
set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Change to the script's directory
cd "$SCRIPT_DIR"

# Build the documentation
mdbook build

echo "Documentation built successfully!"
echo "HTML output: book/html/index.html"
echo "Markdown output: book/markdown/"

# Optionally serve the documentation
if [[ "$1" == "--serve" ]]; then
    echo "Starting server on http://localhost:3000"
    mdbook serve --open
fi 