# https://just.systems

test-cli: build-debug
    PATH=`pwd`/target/debug:$PATH clitest tests/*

test-rust:
    cargo test

test: test-cli test-rust

build-debug:
    cargo build --bin stylus

dev: clean-bundle
    #!/usr/bin/env bash
    set -o pipefail
    echo "Creating a temporary instance of Stylus..."
    BUILD_DIR=target/temp-instance
    rm -rf "$BUILD_DIR"
    if [ -n "$CONFIG_DIR" ]; then
        cp -R "$CONFIG_DIR" "$BUILD_DIR"
        mkdir -p "$BUILD_DIR/static" || true
    else
        cargo run --bin stylus --no-default-features -- init "$BUILD_DIR"
    fi
    for file in crates/stylus-ui/web/src \
        crates/stylus-ui/web/stubs \
        crates/stylus-ui/web/devmode \
        crates/stylus-ui/web/stylus.svg \
        crates/stylus-ui/web/index.html \
        crates/stylus-ui/web/import_map.json; do
        ln -s ../../../"$file" "$BUILD_DIR/static/$(basename "$file")"
    done
    cargo run --bin stylus --no-default-features -- run "$BUILD_DIR"

ts-check:
    cd crates/stylus-ui/web && npx tsc --noEmit

clean-bundle:
    rm crates/stylus-ui/src/compiled/* 2>/dev/null || true

bundle: clean-bundle
    #!/usr/bin/env bash
    set -o pipefail
    TEMP_DIR=$(mktemp -d)
    echo "Building Stylus UI crate..."
    cargo build -p stylus-ui --features=from-source-always --target-dir="$TEMP_DIR" 2>/dev/null
    cp "$TEMP_DIR"/debug/build/stylus-ui-*/out/stylus.* crates/stylus-ui/src/compiled/
    ls -l crates/stylus-ui/src/compiled/
    rm -rf "$TEMP_DIR"

release-tag:
    #!/usr/bin/env bash
    set -euf -o pipefail
    VERSION="v$(cargo metadata --format-version=1 | jq -r '.packages[] | select(.name == "stylus") | .version')"
    echo "Creating tag $VERSION..."
    git tag "$VERSION"
    echo "Tag created. Push with: git push origin $VERSION"

publish: bundle
    cargo publish -p stylus-ui --allow-dirty
    cargo publish -p stylus

publish-docker:
    #!/usr/bin/env bash
    set -euf -o pipefail
    VERSION="v$(cargo metadata --format-version=1 | jq -r '.packages[] | select(.name == "stylus") | .version')"
    docker/build.sh "$VERSION"

update-logo:
    cp logo/logo.svg logo/stylus-black-1024x1024.svg
    svgo logo/stylus-black-1024x1024.svg
    cp logo/logo.svg logo/stylus-white-1024x1024.svg
    sed -i '' 's/fill="black"/fill="white"/g' logo/stylus-white-1024x1024.svg
    svgo logo/stylus-white-1024x1024.svg
    cp logo/stylus-black-1024x1024.svg crates/stylus-ui/web/stylus.svg
    