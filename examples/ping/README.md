# Ping Monitor Example

This example demonstrates the ping monitor functionality. It monitors the connectivity to Google's DNS server (8.8.8.8).

## Configuration

The monitor is configured to:
- Ping 8.8.8.8 every 30 seconds
- Wait up to 3 seconds for responses
- Show orange status if ping succeeds but takes more than 500ms
- Show red status if any packets are lost
- Show green status for successful pings under 500ms

## Running

```bash
stylus run examples/ping/config.yaml
```

Then visit http://localhost:8081 to see the monitor status.

## Testing

You can test the monitor configuration with:

```bash
stylus test examples/ping/config.yaml dns-servers
```
