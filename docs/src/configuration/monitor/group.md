# Group Monitor

A group monitor allows a single test script's execution to update the state for
multiple entities. For example, you may be able to scrape the state of multiple
hosts from a single controller, or you may want to monitor the state of multiple
ports on a single switch.

## Configuration

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

## State Output

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

## Example

Here's a complete example of a group monitor that checks the status of multiple network ports:

```yaml
group:
    id: port-{{ index }}
    axes:
        - name: index
          values: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23]
    test:
        interval: 10s
        timeout: 30s
        command: test.sh
```

With a corresponding `test.sh` script:

```bash
#!/bin/bash
set -xeuf -o pipefail
echo '@@STYLUS@@ group.port-0.status.status="yellow"'
echo '@@STYLUS@@ group.port-1.status.status="green"'
echo '@@STYLUS@@ group.port-2.status.status="yellow"'
echo '@@STYLUS@@ group.port-3.status.status="green"'
echo '@@STYLUS@@ group.port-4.status.status="green"'
echo '@@STYLUS@@ group.port-5.status.status="yellow"'
echo '@@STYLUS@@ group.port-6.status.status="yellow"'
if [ $((RANDOM % 2)) -eq 0 ]; then
    echo '@@STYLUS@@ group.port-7.status.status="red"'
else
    echo '@@STYLUS@@ group.port-7.status.status="green"'
fi
```

## Metadata

Group monitors can also set metadata for each individual monitor using the same `@@STYLUS@@` prefix:

```bash
echo '@@STYLUS@@ group.port-0.status.description="Port 0 is experiencing high latency"'
echo '@@STYLUS@@ group.port-0.status.metadata.latency="150ms"'
echo '@@STYLUS@@ group.port-1.status.description="Port 1 is healthy"'
echo '@@STYLUS@@ group.port-1.status.metadata.latency="5ms"'
```
