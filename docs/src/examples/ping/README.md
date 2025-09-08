# Ping Monitoring

The simplest monitor is a ping script. One ping is usually enough for most
cases. You can pass a timeout to ping, but **Stylus** will automatically kill
processes if they run too long.

## Built-in Ping Monitor

Stylus has a [built-in ping monitor](../../configuration/monitor/ping.md) that
can be used to monitor network connectivity.

```yaml
ping:
  host: 8.8.8.8
  interval: 30s
  timeout: 10s
  count: 1
```

## Custom Ping Script

You can also write a custom ping script to test the connectivity to a host.

```yaml
test:
  id: my-ping-host
  interval: 30s
  timeout: 10s
  command: ping.sh
```

With a corresponding `ping.sh` script:

```bash
#!/bin/bash
# Pings the host with the same name as the monitor's ID
ping -c 1 ${STYLUS_MONITOR_ID}
```

Pinging a custom host:

```bash
#!/bin/bash
ping -c 1 8.8.8.8
```

Pinging with a custom per-packet timeout:

```bash
#!/bin/bash
# -W is in seconds on Linux
ping -c 1 -W 5 ${STYLUS_MONITOR_ID}
```

## Alternatives

For more complex monitoring scenarios, consider using [SSH](../ssh/),
[SNMP](../snmp/), or [HTML/API scraping](../scraping/). 
