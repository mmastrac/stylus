# Standard Monitor

A standard monitor consists of a single test for a single host. This is the most common type of monitor in Stylus.

## Configuration

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

## Example

Here's a simple example of a standard monitor that pings a host:

```yaml
test:
  id: web-server-ping
  interval: 30s
  timeout: 10s
  command: ping.sh
```

With a corresponding `ping.sh` script:

```bash
#!/bin/bash
ping -c 1 -W 5 example.com > /dev/null 2>&1
exit $?
```

## Environment Variables

**Stylus** invokes all test scripts with a special environment variable named `STYLUS_MONITOR_ID`. This may be used
as a convenient way to test multiple monitors using shared scripts. For example, a test script may be configured 
like so:

```bash
ssh $STYLUS_MONITOR_ID my-test-command
```
