#!/bin/bash
set -xeuf -o pipefail
if [ "$#" != "1" ]; then
    echo "Usage: $0 <version>"
    exit 1
fi

VERSION="$1"
echo "Building Docker container for $VERSION..."

docker build --no-cache --platform linux/arm/v6 \
    --build-arg VERSION=$VERSION \
    --build-arg BINARYPLATFORM=linux_arm \
    --build-arg BUILDPLATFORM=arm32v6 \
    --build-arg RUSTPLATFORM=arm-unknown-linux-musleabi \
    -t mmastrac/stylus:latest-arm -f docker/Dockerfile .
docker build --no-cache --platform linux/arm64 \
    --build-arg VERSION=$VERSION \
    --build-arg BINARYPLATFORM=linux_arm64 \
    --build-arg BUILDPLATFORM=arm64v8 \
    --build-arg RUSTPLATFORM=aarch64-unknown-linux-musl \
    -t mmastrac/stylus:latest-arm64 -f docker/Dockerfile .
docker build --no-cache --platform linux/amd64 \
    --build-arg VERSION=$VERSION \
    --build-arg BINARYPLATFORM=linux_amd64 \
    --build-arg BUILDPLATFORM=amd64 \
    --build-arg RUSTPLATFORM=x86_64-unknown-linux-musl \
    -t mmastrac/stylus:latest-x86_64 -f docker/Dockerfile .

docker push mmastrac/stylus:latest-arm
docker push mmastrac/stylus:latest-arm64
docker push mmastrac/stylus:latest-x86_64

# TBH I don't fully understand manifests, but this seems to work
rm -rf ~/.docker/manifests/docker.io_mmastrac_stylus-latest
docker manifest create mmastrac/stylus:latest \
    mmastrac/stylus:latest-arm \
    mmastrac/stylus:latest-arm64 \
    mmastrac/stylus:latest-x86_64

docker manifest annotate --os linux --arch arm --variant v6 mmastrac/stylus:latest mmastrac/stylus:latest-arm
docker manifest annotate --os linux --arch arm64 --variant v8 mmastrac/stylus:latest mmastrac/stylus:latest-arm64
docker manifest annotate --os linux --arch amd64 mmastrac/stylus:latest mmastrac/stylus:latest-x86_64

docker manifest push mmastrac/stylus:latest

rm -rf ~/.docker/manifests/docker.io_mmastrac_stylus-$VERSION
docker manifest create mmastrac/stylus:$VERSION \
    mmastrac/stylus:latest-arm \
    mmastrac/stylus:latest-arm64 \
    mmastrac/stylus:latest-x86_64

docker manifest annotate --os linux --arch arm --variant v6 mmastrac/stylus:$VERSION mmastrac/stylus:latest-arm
docker manifest annotate --os linux --arch arm64 --variant v8 mmastrac/stylus:$VERSION mmastrac/stylus:latest-arm64
docker manifest annotate --os linux --arch amd64 mmastrac/stylus:$VERSION mmastrac/stylus:latest-x86_64

docker manifest push mmastrac/stylus:$VERSION
