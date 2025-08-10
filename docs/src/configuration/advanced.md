# Advanced Configuration

## Metadata

A test script may update metadata for the monitor, including the built-in status and description fields. These commands start with the prefix `@@STYLUS@@` and may be output to standard output or standard error.

An example of metadata update commands is shown below:

```
echo '@@STYLUS@@ status.description="Custom (yellow)"'
echo '@@STYLUS@@ status.status="yellow"'
echo '@@STYLUS@@ status.metadata.key="value1"'
```

These may be referenced via standard interpolation, such as `{{monitor.status.metadata.key}}`.

## Environment variables

**Stylus** invokes all test scripts with a special environment variable named `STYLUS_MONITOR_ID`. This may be used
as a convenient way to test multiple monitors using shared scripts. For example, a test script may be configured 
like so:

```bash
ssh $STYLUS_MONITOR_ID my-test-command
```
