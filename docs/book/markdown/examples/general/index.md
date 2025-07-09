# General Tips

There are several approaches you can take to monitoring with Stylus. This
section covers general best practices and tips for writing effective monitor
scripts.

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

## Testing Your Configurations

As monitor scripts using metadata can be somewhat tricky to get right, *Stylus*
includes a `test` command-line argument to allow you to develop your test
script in a slightly more interactive manner. The output from `test` will
include the test script's stdout and stderr streams, plus the parsed monitor
state as JSON, and the final rendered CSS.
