# Monitor Configuration

Monitor configurations define how **Stylus** tests your infrastructure components. Each monitor consists of a test script that runs on a schedule and reports the status back to **Stylus**.

## Monitor Types

Stylus supports several types of monitors:

- **[Standard Monitor](standard.md)** - Single test for a single host
- **[Group Monitor](group.md)** - Single script that updates multiple monitors
- **[SNMP Monitor](snmp.md)** - Network device monitoring via SNMP
- **[Ping Monitor](ping.md)** - Network connectivity monitoring via ping

## Logging

Output from the test's standard output and standard error streams are captured
and available from the logging endpoint.

## Monitor States

The state of a monitor is determined by the return value of the test script or
manually set by scripts/expressions. The six states, in order of precedence,
are:

| State |  | Description | How it's set |
|-------|--------|-------------|--------------|
| Red | 🔴 | Tests that fail by returning a value other than zero | Automatic (exit code ≠ 0) |
| Orange | 🟠 | Warning state | Manual (scripts/expressions) |
| Yellow | 🟡 | A test that has timed out | Automatic (timeout) |
| Blue | 🔵 | Highlight state | Manual (scripts/expressions) |
| Green | 🟢 | Tests that return zero (success) | Automatic (exit code = 0) |
| Blank | ⚪ | A test that has not run or completed yet | Automatic (initial state) |

## Metadata

Tests scripts may also set metadata associated with the run. More information on
this is available in [Advanced Configuration](../advanced.md). 

## Testing Your Configurations

Since monitor scripts with metadata can be tricky to get right, **Stylus** includes a [`stylus test` command](../../getting-started/stylus-test.md) that lets you develop your test script interactively. The output shows your script's stdout and stderr, plus the parsed monitor state as JSON, and the final rendered CSS.
