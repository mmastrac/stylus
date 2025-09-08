# SNMP Monitor

If you want to monitor network devices, you can often use
[SNMP](https://en.wikipedia.org/wiki/Simple_Network_Management_Protocol) to
extract information from the device. SNMP allows you to query the device for
information about its status using OIDs (Object Identifiers), which are roughly
standardized across different devices.

This is particularly useful for monitoring switches, routers, and other network
infrastructure.

See the manual for your networking device for the appropriate OIDs to use, or
reference one of the following resources: 

 - <https://ldapwiki.com/wiki/Wiki.jsp?page=SNMP>
 - <https://networklessons.com/cisco/ccie-routing-switching/introduction-to-snmp>

The SNMP monitor works like a [group monitor](group.md), but it uses SNMP to
query the network device and create children for each detected interface.

## Requirements

The SNMP monitor requires the `snmpwalk` or `snmpbulkwalk` command to be
installed on the system. These are typically available in the `net-snmp`
package.

## Configuration

Interfaces are first filtered by the `include` and `exclude` conditions. If
`include` is true and `exclude` is false, the interface is included in the
monitor.

The `red`, `orange`, `yellow`, `blue` and `green` conditions are evaluated using
the [expressions](../expressions.md) language. The first condition that is true
will determine the status of the interface.

By default, the SNMP monitor will show interfaces as green if they have
`ifOperStatus` and `ifAdminStatus` set to `"up"`, and blank if either of those
is not `"up"`.

```yaml
snmp:
  # The ID pattern for this monitor. Uses interpolation from interface index.
  id: router-{{ index }}

  # How often the SNMP queries are performed
  interval: 60s

  # How long to wait for SNMP responses
  timeout: 30s

  # (optional) Filter to include certain interfaces (default: "true")
  include: |
    ifType == 'ethernetCsmacd'

  # (optional) Filter to exclude certain interfaces (default: "false")
  exclude: |
    contains(ifDescr, 'Loopback')

  # (optional) Condition that determines when the monitor should be red/error (default: "false")
  red: |
    ifOperStatus == "up" and ifSpeed < 1000000000

  # (optional) Condition that determines when the monitor should be orange/warning (default: "false")
  orange: |
    ifOperStatus == "up" and ifSpeed < 1000000000

  # (optional) Condition that determines when the monitor should be yellow/timeout (default: "false")
  yellow: |
    false

  # (optional) Condition that determines when the monitor should be blue/highlight (default: "false")
  blue: |
    ifOperStatus == "up" and ifSpeed > 1000000000

  # (optional) Condition that determines when the monitor should be green (default: "ifOperStatus == 'up' and ifAdminStatus == 'up'")
  green: |
    ifOperStatus == "up" and ifAdminStatus == "up"

  # SNMP target configuration
  target:
    host: 192.168.1.254
    port: 161 # optional, defaults to 161
    version: 2 # optional, defaults to 2 (1, 2, or 3)
    community: public # for SNMP v1/v2c
    # For SNMP v3:
    # username: myuser
    # auth_protocol: SHA  # optional
    # auth_password: myauthpass  # optional
    # privacy_protocol: AES  # optional
    # privacy_password: myprivpass  # optional
    bulk: true # optional, defaults to true
```

## Parameters

### Required Parameters

| Parameter | Description |
|-----------|-------------|
| `id` | The monitor ID pattern using interpolation with `{{ index }}` |
| `interval` | How often to perform SNMP queries |
| `timeout` | SNMP query timeout |
| `target.host` | The IP address or hostname of the SNMP device |

### Optional Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `include` | A filter expression to include certain SNMP interfaces | `"true"` |
| `exclude` | A filter expression to exclude certain SNMP interfaces | `"false"` |
| `red` | A condition that determines when the monitor should show red status | `"false"` |
| `green` | A condition that determines when the monitor should show green status | `"ifOperStatus == 'up' and ifAdminStatus == 'up'"` |
| `target.port` | SNMP port | `161` |
| `target.version` | SNMP version (1, 2, or 3) | `2` |
| `target.community` | SNMP community string (for v1/v2c) | `"public"` |
| `target.username` | SNMP username (for v3) | - |
| `target.auth_protocol` | Authentication protocol (SHA, MD5, etc.) | - |
| `target.auth_password` | Authentication password | - |
| `target.privacy_protocol` | Privacy protocol (AES, DES, etc.) | - |
| `target.privacy_password` | Privacy password | - |
| `target.bulk` | Use bulk SNMP operations | `true` |

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

The SNMP monitor automatically queries the SMTP `ifTable` table, and makes the
OIDs available in expressions. The most useful ones you might want to use are:

| Field            | OID/Variable    | Description                                                                                       |
| ---------------- | --------------- | ------------------------------------------------------------------------------------------------- |
| Status           | `ifOperStatus`  | Operational status of interface (`up` or `down`, ie: copper connected, fiber connected, etc.)     |
| Admin Status     | `ifAdminStatus` | Administrative status of interface (`up` or `down`, ie: enabled or disabled by the administrator) |
| Type             | `ifType`        | Type of interface (`ethernetCsmacd`, `loopback`, `other`, etc.)                                   |
| Speed            | `ifSpeed`       | Speed of interface                                                                                |
| Description      | `ifDescr`       | Description of interface                                                                          |
| Name             | `ifName`        | Name of interface                                                                                 |
| Alias            | `ifAlias`       | Alias of interface                                                                                |
| MTU              | `ifMtu`         | Maximum Transmission Unit                                                                         |
| Physical Address | `ifPhysAddress` | Physical (MAC) address                                                                            |

You can see the OIDs available in the `ifTable` table on your specific device
with the `snmptable` command:

```
# Print the headers of the ifTable table
snmptable -Ch -v 2c -c public 192.168.1.1 ifTable | head -1
```

## SNMP Versions

The SNMP monitor supports SNMP v1, v2c, and v3. The default is to use v2c with
the `public` community string. Bulk operations are enabled by default, but can
be disabled for older devices.

### SNMP v1/v2c

```yaml
snmp:
  # ... other configuration ...
  target:
    host: 192.168.1.254
    version: 2 # or 1
    community: public
```

### SNMP v3

```yaml
snmp:
  # ... other configuration ...
  target:
    host: 192.168.1.254
    version: 3
    username: myuser
    auth_protocol: SHA
    auth_password: myauthpass
    privacy_protocol: AES
    privacy_password: myprivpass
```
