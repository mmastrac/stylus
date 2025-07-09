# General Tips

There are several approaches you can take to monitoring with Stylus. This
section covers general best practices and tips for writing effective monitor
scripts.

## `STYLUS_MONITOR_ID`

The `STYLUS_MONITOR_ID` environment variable is set by **Stylus** to the monitor's
ID when running a monitor script. This allows you to write monitor scripts that
can be re-used across multiple monitors.

```bash
#!/bin/bash
set -xeuf -o pipefail
# Check the health of a service running on the monitor
curl --fail http://$STYLUS_MONITOR_ID:8080/health | jq --raw-output '.status'
```

## Safe Scripting

Because monitor scripts may have a large number of moving parts, consider using
[safe shell scripting](https://sipb.mit.edu/doc/safe-shell/) techniques to
ensure that any failure of any kind will return an error code.

In addition `set -x` can be useful to print all commands that run as part of a
monitor script. These are available in the logging endpoints and will show you
the expansion of environment variables.

```bash
#!/bin/bash
set -xeuf -o pipefail
```

## Script-relative Paths

The `dirname` command can be used to get the directory of the script that is
running. This can be useful to locate configuration files or other resources
that are needed by the script, which allows you to run the script from any
directory outside of **Stylus**.

```bash
#!/bin/bash
set -xeuf -o pipefail
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

CONFIG_FILE=$DIR/config.yaml
```

## Testing Your Configurations

As monitor scripts using metadata can be somewhat tricky to get right,
**Stylus** includes a [`test` command-line
argument](../getting-started/stylus-test.md) to allow you to develop your test
script in a slightly more interactive manner.

The output from `stylus test` will include the test script's stdout and stderr
streams, as well as the parsed monitor state as JSON, and the final rendered
CSS.
