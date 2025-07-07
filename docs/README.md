# Stylus Documentation

This directory contains the Stylus documentation built with [mdBook](https://rust-lang.github.io/mdBook/).

## About Stylus

**Stylus** (_style + status_) is a lightweight status page for home
infrastructure. Configure a set of bash scripts that test the various parts of
your infrastructure, set up HTML/SVG with a diagram of your network, and stylus
will generate you a dynamic stylesheet to give you a visual overview of the
current state.

## Building the Documentation

To build the documentation:

```bash
cd docs
mdbook build
```

Or use the provided build script:

```bash
cd docs
./build.sh
```

To build and serve the documentation locally:

```bash
cd docs
./build.sh --serve
```

## Documentation Structure

The documentation is organized as follows:

- [Introduction](src/wiki-home.md) - Main documentation home page
- [Getting Started](src/wiki-getting-started.md) - Quick start guide
- [Running Stylus](src/wiki-running-stylus.md) - How to run Stylus
- [Creating a Monitor Page](src/wiki-creating-a-monitor-page.md) - How to create monitor pages
- [Configuration](src/wiki-configuration.md) - Configuration options and examples
- [Advanced Configuration](src/wiki-advanced-configuration.md) - Advanced configuration features
- [Examples](src/wiki-examples.md) - Example configurations and use cases

## Output

After building, the documentation will be available in:

- **HTML**: `book/html/index.html` - Full HTML documentation with navigation and search
- **Markdown**: `book/markdown/` - Plain markdown output

## Source

The documentation source files are in the `src/` directory. The structure is defined in `src/SUMMARY.md`.
