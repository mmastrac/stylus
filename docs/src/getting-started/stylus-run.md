# stylus run

Run **Stylus** (default command)

## Usage

```bash
stylus run [OPTIONS] [FILE]
```

## Arguments

- `[FILE]` - The configuration file (or directory)

## Options

- `--dry-run` - Dry run the configuration (everything except running the server)
- `-v, --verbose...` - Pass multiple times to increase the level of verbosity (overwritten by STYLUS_LOG)
- `-h, --help` - Print help

The `stylus run` command starts the **Stylus** server and begins monitoring your infrastructure. This is the main command you'll use to run **Stylus** in production.

The command loads your configuration, starts the HTTP server, and begins running your monitor tests on their schedules. It also serves your status page and provides API endpoints for the current monitor states.

## Examples

```bash
# Run with a specific config file
stylus run config.yaml

# Run using a directory (will look for config.yaml inside)
stylus run ~/my-stylus/

# Run with verbose output
stylus run -v ~/my-stylus/

# Dry run - test configuration without starting server
stylus run --dry-run ~/my-stylus/
```

If you specify a directory instead of a file, **Stylus** will look for `config.yaml` inside that directory. This is the most common way to run it.

## Dry Run Mode

The `--dry-run` option tests your configuration without starting the server. This is useful for validating configuration syntax locally or in a CI/CD pipeline.

## Server Endpoints

Once running, the server provides several endpoints:

- `/` - Main status page
- `/status.json` - JSON API with current monitor states
- `/style.css` - Dynamic CSS with current monitor states
- `/log/<monitor-id>` - Log output for specific monitors

## Stopping the Server

Use `Ctrl+C` to stop the server gracefully. **Stylus** will clean up any running monitor processes. 