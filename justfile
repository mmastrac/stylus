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
    mkdir -p crates/stylus-ui/src/compiled/
    cp "$TEMP_DIR"/debug/build/stylus-ui-*/out/stylus.* crates/stylus-ui/src/compiled/
    ls -l crates/stylus-ui/src/compiled/
    rm -rf "$TEMP_DIR"

release-tag:
    #!/usr/bin/env bash
    set -euf -o pipefail
    VERSION="v$(cargo metadata --format-version=1 | jq -r '.packages[] | select(.name == "stylus") | .version')"
    echo "Creating tag $VERSION..."
    git tag "$VERSION" || (echo "Tag already exists, run git tag -d $VERSION" && exit 1)
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

screenshot-examples: build-debug
    #!/usr/bin/env bash
    set -euo pipefail
    
    # Ensure screenshots directory exists
    mkdir -p docs/src/screenshots
    
    # Find Chrome/Chromium executable
    CHROME=""
    for browser in "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome" \
                   "/Applications/Chromium.app/Contents/MacOS/Chromium" \
                   "google-chrome" \
                   "chromium" \
                   "chromium-browser"; do
        if command -v "$browser" >/dev/null 2>&1 || [ -x "$browser" ]; then
            CHROME="$browser"
            break
        fi
    done
    
    if [ -z "$CHROME" ]; then
        echo "Error: Chrome or Chromium not found. Please install Chrome or Chromium."
        exit 1
    fi
    
    echo "Using Chrome/Chromium at: $CHROME"
    
    # Get list of example directories
    EXAMPLES=($(find examples -maxdepth 1 -type d -not -path examples | sort))
    for example_dir in "${EXAMPLES[@]}"; do
        example_name=$(basename "$example_dir")
        echo "Processing example: $example_name"
        
        # Start Stylus in background
        echo "Starting Stylus for $example_name..."
        target/debug/stylus run "$example_dir" &
        STYLUS_PID=$!
        
        echo "Waiting for server to start..."
        for i in {1..30}; do
            if curl -s http://localhost:8000 >/dev/null 2>&1; then
                echo "Server is ready!"
                break
            fi
            if [ $i -eq 30 ]; then
                echo "Timeout waiting for server to start for $example_name"
                kill $STYLUS_PID 2>/dev/null || true
                continue 2
            fi
            sleep 1
        done
        
        sleep 10

        # Take screenshot with delay
        echo "Taking screenshot for $example_name..."
        screenshot_path="docs/src/screenshots/examples/${example_name}.png"
        "$CHROME" \
            --headless=new \
            --disable-gpu \
            --screen-info={1600x1200} \
            --window-size=1200,1000 \
            --timeout=10000 \
            --virtual-time-budget=10000 \
            --force-device-scale-factor=1 \
            --force-color-profile=srgb \
            --force-prefers-color-scheme=light \
            --screenshot="$screenshot_path" \
            "http://localhost:8000"
        
        # Trim constant color from edges
        echo "Trimming screenshot for $example_name..."
        if command -v magick >/dev/null 2>&1; then
            magick "$screenshot_path" -trim "$screenshot_path"
        elif command -v convert >/dev/null 2>&1; then
            convert "$screenshot_path" -trim "$screenshot_path"
        else
            echo "Warning: ImageMagick not found, skipping trim for $example_name"
        fi
        
        # Stop Stylus
        echo "Stopping Stylus..."
        kill $STYLUS_PID 2>/dev/null || true
        
        # Wait a moment for cleanup
        sleep 2
        
        echo "Screenshot saved: docs/src/screenshots/${example_name}.png"
        echo "---"
    done
    
    echo "All screenshots completed!"
    echo "Screenshots saved in: docs/src/screenshots/"
