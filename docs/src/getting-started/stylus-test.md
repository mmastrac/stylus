# stylus test

Runs the given test immediately and displays the status of the given monitor after it completes.

## Usage

```bash
stylus test [OPTIONS] --monitor <MONITOR> <FILE>
```

## Arguments

- `<FILE>` - The configuration file

## Options

- `-m, --monitor <MONITOR>` - The test to run
- `-v, --verbose...` - Pass multiple times to increase the level of verbosity (overwritten by STYLUS_LOG)
- `-h, --help` - Print help

## Description

The `stylus test` command allows you to run a specific monitor test immediately and see its output without starting the full server.

## Example

```bash
# Test a monitor named "web-server" using config.yaml
stylus test --monitor web-server config.yaml

# Test with verbose output
stylus test -v --monitor web-server config.yaml

# Test using a directory (will look for config.yaml inside)
stylus test --monitor web-server ~/my-stylus/
```

## Output Format

The command outputs three sections:

1. _Monitor Log_: Raw output from the test script execution
2. _State_: JSON representation of the monitor's current state
3. _CSS_: Generated CSS rules for the monitor
