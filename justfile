# https://just.systems

test-cli: build-debug
    PATH=`pwd`/target/debug:$PATH clitest --quiet tests/*

test-rust:
    cargo test

test: test-cli test-rust

build-debug:
    cargo build --bin stylus

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
    cd crates/stylus-ui/ \
      && cp web/src/style.css src/compiled/stylus.css \
      && deno bundle --config web/deno.json --minify --platform browser \
        --output src/compiled/stylus.js --sourcemap=external web/src/app.tsx \
      && gzip -9 --force src/compiled/stylus.js.map

release-tag:
    #!/usr/bin/env bash
    VERSION="v$(cargo metadata --format-version=1 | jq -r '.packages[] | select(.name == "stylus") | .version')"
    echo "Creating tag $VERSION..."
    git tag "$VERSION"
    echo "Tag created. Push with: git push origin $VERSION"
