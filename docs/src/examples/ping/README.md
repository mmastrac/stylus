# Ping Monitoring

The simplest monitor is a ping script. One ping is usually enough for most cases. You can pass a timeout to ping, but **Stylus** will automatically kill processes if they run too long.

## Basic Ping Script

```bash
#!/bin/bash
ping -c 1 ${STYLUS_MONITOR_ID}
```

## Ping with Custom Host

```bash
#!/bin/bash
ping -c 1 8.8.8.8
```

## Ping with Timeout

```bash
#!/bin/bash
ping -c 1 -W 5 ${STYLUS_MONITOR_ID}
```

## When to Use Ping

Ping monitoring is ideal for:
- Basic connectivity testing
- Simple network reachability checks
- Quick health checks for network devices
- Testing internet connectivity

For more complex monitoring scenarios, consider using [SSH](../ssh/), [SNMP](../snmp/), or [HTML/API scraping](../scraping/). 