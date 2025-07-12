# https://just.systems

default:
    echo 'Hello, world!'

dev:
    #!/usr/bin/env bash
    set -o pipefail
    echo "Creating a temporary instance of Stylus..."
    BUILD_DIR=target/temp-instance
    rm -rf "$BUILD_DIR"
    cargo run --bin stylus --no-default-features -- init "$BUILD_DIR"
    for file in crates/stylus-ui/web/src \
        crates/stylus-ui/web/index.html \
        crates/stylus-ui/web/import_map.json \
        crates/stylus-ui/web/babel-module-loader.js; do
        ln -s ../../../"$file" "$BUILD_DIR/static/$(basename "$file")"
    done
    cargo run --bin stylus --no-default-features -- run "$BUILD_DIR"

ts-check:
    cd crates/stylus-ui/web && npx tsc --noEmit

bundle:
    cd crates/stylus-ui/web && mkdir -p build \
      && deno bundle --minify --platform browser --output build/app.js --sourcemap=external src/app.tsx
