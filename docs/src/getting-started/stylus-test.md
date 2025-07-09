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

The `stylus test` command allows you to run a specific monitor test immediately and see its output without starting the full server. This is useful for:

- Debugging monitor scripts
- Testing monitor configurations
- Verifying monitor behavior
- Development and troubleshooting

The command will:
1. Run the specified monitor's test script
2. Display the test output (stdout/stderr)
3. Show the parsed monitor state as JSON
4. Display the generated CSS for the monitor

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

1. **Monitor Log**: Raw output from the test script execution
2. **State**: JSON representation of the monitor's current state
3. **CSS**: Generated CSS rules for the monitor

## Use Cases

- **Development**: Test monitor scripts during development
- **Debugging**: Identify issues with monitor configurations
- **Validation**: Verify monitor behavior before deployment
- **Troubleshooting**: Isolate problems with specific monitors 