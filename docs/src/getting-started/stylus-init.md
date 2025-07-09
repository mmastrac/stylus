# stylus init

Initialize a new stylus directory with default configuration and monitor setup.

## Usage

```bash
stylus init [OPTIONS] <DIRECTORY>
```

## Arguments

- `<DIRECTORY>` - The directory to initialize

## Options

- `-v, --verbose...` - Pass multiple times to increase the level of verbosity (overwritten by STYLUS_LOG)
- `-h, --help` - Print help

## Description

The `stylus init` command creates a new Stylus project directory with the following structure:

```
<DIRECTORY>/
├── config.yaml          # Main configuration file
├── monitor.d/           # Monitor directory
│   └── monitor/         # Default monitor
│       ├── config.yaml  # Monitor configuration
│       └── test.sh      # Test script
└── static/              # Static files directory
    └── README.md        # Placeholder for index.html
```

## Example

```bash
# Initialize a new Stylus project in ~/my-stylus
stylus init ~/my-stylus

# Initialize with verbose output
stylus init -v ~/my-stylus
```

## What Gets Created

- **config.yaml**: Default server configuration with port 8000
- **monitor.d/monitor/config.yaml**: Default monitor with 30-second interval and 10-second timeout
- **monitor.d/monitor/test.sh**: Executable test script with placeholder content
- **static/**: Directory for your HTML/SVG files

After initialization, you can start the server with `stylus run <DIRECTORY>`. 