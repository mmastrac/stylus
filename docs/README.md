# Stylus Documentation

This directory contains the Stylus documentation built with [mdBook](https://rust-lang.github.io/mdBook/).

## Building the Documentation

To build the documentation:

```bash
cd docs
mdbook build
```

Or use the provided build script:

```bash
docs/build.sh
```

To build and serve the documentation locally:

```bash
docs/build.sh --serve
```

## Source

The documentation source files are in the `src/` directory. The structure is defined in `src/SUMMARY.md`.
