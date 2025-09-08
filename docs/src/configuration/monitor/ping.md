# Ping Monitor

The ping monitor uses the system `ping` command to check network connectivity to
a host. It measures round-trip time and packet loss, making it useful for
monitoring network latency and availability.

## Configuration

The ping monitor evaluates conditions using the [expressions](../expressions.md) language.

By default, the ping monitor will show:

- **Green** if there's no packet loss and the round-trip time is within the
  warning timeout
- **Orange** if there's no packet loss but the round-trip time exceeds the
  warning timeout, or if there was partial packet loss
- **Yellow** if the ping command timed out
- **Red** if there was complete packet loss

```yaml
ping:
  # The host to ping (IP address or hostname)
  host: 8.8.8.8
  
  # How often to perform the ping test
  interval: 60s
  
  # How long to wait for ping responses before timing out
  timeout: 5s
  
  # (optional) Warning threshold for round-trip time (default: 1s)
  warning_timeout: 1s
  
  # (optional) Number of ping packets to send (default: 1)
  count: 1

  # (optional) Condition that determines when the monitor should be red/error (default: "lost == count")
  red: |
    lost == count
  
  # (optional) Condition that determines when the monitor should be orange/warning (default: "lost > 0 or (lost == 0 and rtt_max > warning_timeout)")
  orange: |
    lost > 0 or (lost == 0 and rtt_max > warning_timeout)
  
  # (optional) Condition that determines when the monitor should be green (default: "lost == 0")
  green: |
    lost == 0
  
  # (optional) Condition that determines when the monitor should be blue/highlight (default: "false")
  blue: |
    false
  
  # (optional) Condition that determines when the monitor should be yellow/timeout (default: "false")
  yellow: |
    false
```

## Parameters

### Required Parameters

| Parameter | Description |
|-----------|-------------|
| `host` | The IP address or hostname to ping |
| `interval` | How often to perform ping tests |
| `timeout` | How long to wait for ping responses |

### Optional Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `warning_timeout` | Round-trip time threshold for orange status | `1s` |
| `count` | Number of ping packets to send | `1` |
| `red` | Condition for red status | `"lost == count"` |
| `orange` | Condition for orange status | `"lost > 0 or (lost == 0 and rtt_max > warning_timeout)"` |
| `green` | Condition for green status | `"lost == 0"` |
| `blue` | Condition for blue status | `"false"` |
| `yellow` | Condition for yellow status | `"false"` |

### Expression variables

| Variable | Description |
|----------|-------------|
| `count` | Number of ping packets sent |
| `lost` | Number of packets lost |
| `rtt_avg` | Average round-trip time in microseconds |
| `rtt_min` | Minimum round-trip time in microseconds |
| `rtt_max` | Maximum round-trip time in microseconds |
| `warning_timeout` | The configured warning timeout value in microseconds |

## Example

Ping Google's DNS server using the default settings:

```yaml
ping:
  host: 8.8.8.8
```

Ping Google's DNS server with custom settings:

```yaml
ping:
  host: 8.8.8.8
  interval: 30s
  timeout: 10s
  warning_timeout: 500ms
  count: 3
  red: |
    lost > 0
  orange: |
    lost == 0 and rtt_min > warning_timeout
```

This monitor will:

- Ping Google's DNS server (8.8.8.8) every 30 seconds with three packets
- Wait up to 10 seconds for responses
- Show orange/warning status if the shortest round-trip time exceeds 500ms (ie: all pings were slow)
- Show red/error status if any packets are lost

## Requirements

The ping monitor requires the `ping` command to be available on the system. This
is typically installed by default on most Unix-like systems.
