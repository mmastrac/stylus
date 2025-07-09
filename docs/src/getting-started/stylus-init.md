# stylus init

Initialize a new **Stylus** project directory with default [configuration](../configuration/server/) and [monitor](../configuration/monitor/) setup.

## Usage

```bash
stylus init [OPTIONS] <DIRECTORY>
```

## Arguments

- `<DIRECTORY>` - The directory to initialize

## Options

- `-v, --verbose...` - Pass multiple times to increase the level of verbosity (overwritten by STYLUS_LOG)
- `-h, --help` - Print help

The `stylus init` command creates a new **Stylus** project directory with the following structure:

## What Gets Created

```
<DIRECTORY>/
├── config.yaml          # Main configuration file
├── monitor.d/           # Monitor directory
│   └── monitor/         # Default monitor with id "monitor"
│       ├── config.yaml  # Monitor "monitor" configuration
│       └── test.sh      # Test script for "monitor"
└── static/              # Static files directory
    └── README.md        # Placeholder for static files
```

## Example

```bash
# Initialize a new Stylus project in ~/my-stylus
stylus init ~/my-stylus
```

After initialization, you can start the server with `stylus run <DIRECTORY>`. 
