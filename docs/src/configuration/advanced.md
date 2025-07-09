# Advanced Configuration

## Group Monitors

A group monitor allows a single test script's execution to update the state for
multiple entities. For example, you may be able to scrape the state of multiple
hosts from a single controller, or you may want to monitor the state of multiple
ports on a single switch.

```yaml
group:
    # The ID pattern for this group. This ID must use interpolation from axis values to generate a set of
    # globally unique IDs. 
    id: port-{{ index }}

    # The configuration axes.
    axes:
        # The Axis name and a list of values
        - name: index
          values: [0, 1, 2, 3, 4, 5, 6, 7]

    # A standard monitor configuration (see the Standard Monitor description)
    test:
        interval: 60s
        timeout: 30s
        command: test.sh
```

The group's test script is unique in that it must output state-modifying commands to its standard output. Each
of these state-modifying commands starts with the prefix `@@STYLUS@@`.

```bash
echo '@@STYLUS@@ group.port-0.status.status="yellow"'
echo '@@STYLUS@@ group.port-1.status.status="green"'
echo '@@STYLUS@@ group.port-2.status.status="yellow"'
echo '@@STYLUS@@ group.port-3.status.status="green"'
echo '@@STYLUS@@ group.port-4.status.status="green"'
echo '@@STYLUS@@ group.port-5.status.status="yellow"'
echo '@@STYLUS@@ group.port-6.status.status="yellow"'
echo '@@STYLUS@@ group.port-7.status.status="red"'
```

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
