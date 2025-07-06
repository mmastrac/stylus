#!/bin/bash
set -xeuf -o pipefail
if [ "$#" != "1" ]; then
    echo "Usage: $0 v<version>"
    exit 1
fi

if [[ ! "$1" =~ ^v ]]; then
    echo "Error: Version must start with 'v'"
    exit 1
fi

VERSION="$1"

# Ensure buildx is available
if ! docker buildx version >/dev/null 2>&1; then
    echo "docker buildx is not available. Please install Docker Buildx."
    exit 1
fi

# Ensure a buildx builder exists and is active
if ! docker buildx inspect multiarch-builder >/dev/null 2>&1; then
    docker buildx create --name multiarch-builder --use
else
    docker buildx use multiarch-builder
fi

echo "Building and pushing multi-arch Docker images for $VERSION..."

docker buildx build \
    --platform linux/amd64,linux/arm64,linux/arm/v6 \
    --build-arg VERSION=$VERSION \
    --push \
    -t mmastrac/stylus:latest \
    -t mmastrac/stylus:$VERSION \
    -f docker/Dockerfile .

echo "Build and push complete!"
