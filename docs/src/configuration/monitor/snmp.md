# SNMP Monitor

SNMP monitors allow you to monitor network devices using the Simple Network Management Protocol (SNMP). This is particularly useful for monitoring switches, routers, and other network infrastructure.

The SNMP monitor works like a [group monitor](group.md), but it uses SNMP to
query the network device and create children for each detected interface.

## Configuration

```yaml
snmp:
  # The ID pattern for this monitor. Uses interpolation from axis values.
  id: router-{{ index }}
  
  # How often the SNMP queries are performed
  interval: 60s
  
  # How long to wait for SNMP responses
  timeout: 30s
  
  # (optional) Filter to exclude certain interfaces or components
  exclude: |
    ifType != 'ethernetCsmacd'
  
  # (optional) Condition that determines when the monitor should be red
  red: |
    ifOperStatus == "up" and ifSpeed < 1000000000
  
  # SNMP target configuration
  target:
    host: 192.168.1.254
    community: public
```

## Parameters

### Required Parameters

- **`id`**: The monitor ID pattern using interpolation
- **`interval`**: How often to perform SNMP queries
- **`timeout`**: SNMP query timeout
- **`target.host`**: The IP address or hostname of the SNMP device
- **`target.community`**: The SNMP community string

### Optional Parameters

- **`exclude`**: A filter expression to exclude certain SNMP objects
- **`red`**: A condition that determines when the monitor should show red status
- **`yellow`**: A condition that determines when the monitor should show yellow status

## Example

Here's a complete example of an SNMP monitor for a network switch:

```yaml
snmp:
  id: switch-{{ index }}
  interval: 60s
  timeout: 30s
  exclude: |
    ifType != 'ethernetCsmacd'
  red: |
    ifOperStatus == "up" and ifSpeed < 1000000000
  target:
    host: 192.168.1.254
    community: public
```

This monitor will:
- Query the switch at 192.168.1.254 every 60 seconds
- Only monitor Ethernet interfaces (exclude other interface types)
- Show red status if an interface is up but running at less than 1Gbps
- Use the "public" community string for SNMP authentication

## SNMP OIDs

The SNMP monitor automatically queries common SNMP OIDs for network interfaces:

- **Interface Status**: `ifOperStatus`
- **Interface Type**: `ifType`
- **Interface Speed**: `ifSpeed`
- **Interface Description**: `ifDescr`
- **Interface Name**: `ifName`

## Status Conditions

You can define custom conditions for different status colors:

```yaml
snmp:
  # ... other configuration ...
  
  # Show red when interface is down
  red: |
    ifOperStatus == "down"
  
  # Show yellow when interface speed is below 1Gbps
  yellow: |
    ifOperStatus == "up" and ifSpeed < 1000000000
```

## Security Considerations

- Use SNMPv3 with authentication when possible
- Avoid using the default "public" community string in production
- Consider using SNMP communities with read-only access
- Restrict SNMP access to specific IP addresses on your network devices
