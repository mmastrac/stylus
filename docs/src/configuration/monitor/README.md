# Monitor Configuration

Monitor configurations define how **Stylus** tests your infrastructure components. Each monitor consists of a test script that runs on a schedule and reports the status back to **Stylus**.

## Standard Monitor

A standard monitor consists of a single test for a single host.

```yaml
test:
  # (optional) The internal ID to use for this test. If omitted, the ID is inferred from the monitor directory's name.
  id: foo
  # How often the test is run. The interval restarts from the last success or failure of the test.
  interval: 60s
  # How long the script will be given to run before it is killed.
  timeout: 30s
  # The test command to run, relative to the monitor directory. The PATH is not used and the file must be
  # directly executable.
  command: test.sh
```

## Logging

Output from the test's standard output and standard error streams are captured
and available from the logging endpoint.

## Monitor States

The state of a monitor is determined by the return value of the test script.

- Blank: A test that has not run or completed yet
- Yellow: A test that has timed out
- Red: Tests that fail by returning a value other than zero
- Green: Tests that return zero (success)

## Group Monitors

A group may be configured such that a single script may update states for multiple monitors. See [Advanced Configuration](../advanced.md) for examples of configuring such a group monitor.

## Metadata

Tests scripts may also set metadata associated with the run. More information on
this is available in [Advanced Configuration](../advanced.md). 

## Testing Your Configurations

Since monitor scripts with metadata can be tricky to get right, **Stylus** includes a [`stylus test` command](../../getting-started/stylus-test.md) that lets you develop your test script interactively. The output shows your script's stdout and stderr, plus the parsed monitor state as JSON, and the final rendered CSS.
