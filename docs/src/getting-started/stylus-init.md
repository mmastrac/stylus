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

After initialization, you can start the server with `stylus run <DIRECTORY>`. 
