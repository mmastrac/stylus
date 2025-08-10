# SNMP Monitoring

SNMP (Simple Network Management Protocol) is a useful way to write more complex
checks for network devices, but the output of the tools requires some massaging.

Stylus has a [built-in SNMP monitor](../../configuration/monitor/snmp.md) that
can be used to monitor network devices, but in some cases you may want to write
a custom script to monitor SNMP devices for more complex checks.

## Basic SNMP Check

This performs a simple SNMP "ping" to the device, asking for its `sysDescr` and
`sysUpTime` OIDs (system description and uptime respectively).

```bash
#!/bin/sh
set -xeuf -o pipefail
# Print the SNMP OID for the system description
snmpwalk -v 2c -c public my-network-router 1.3.6.1.2.1.1.1.0
# Print the SNMP OID for the system uptime
snmpwalk -v 2c -c public my-network-router 1.3.6.1.2.1.1.3.0
```

## SNMP Group Monitor

You can use a [group monitor](../../configuration/monitor/group.md) to monitor
multiple devices at once using SNMP.

```bash
snmp_check () {
    local host="$STYLUS_MONITOR_ID"
    ARR=`snmpbulkwalk -OsQ -c public $host ifTable`
    jq -n --arg inarr "${ARR}" '[$inarr | split("\n")
        | .[]
        | capture("(?<key>[^\\.]+)\\.(?<idx>\\d+)\\s+=\\s+(?<value>.*)")
    ] | group_by(.idx) | .[] | from_entries'
}

# Some legacy devices only respond to SNMP v1
snmp_v1_check () {
    local host="$STYLUS_MONITOR_ID"
    ARR=`snmpwalk -v1 -OsQ -c public $host ifTable`
    jq -n --arg inarr "${ARR}" '[$inarr | split("\n")
        | .[]
        | capture("(?<key>[^\\.]+)\\.(?<idx>\\d+)\\s+=\\s+(?<value>.*)")
    ] | group_by(.idx) | .[] | from_entries'
}

snmp_parse () {
    cat - | jq -r '
        # Only parse ethernet ports and omit anything that looks like a vlan port (ending with a .xxxx)
        select(.ifType=="ethernetCsmacd" and (.ifDescr | test("\\.\\d+$") | not)) 
        | "@@STYLUS@@ group.'$STYLUS_MONITOR_ID'-" 
            + .ifIndex 
            + ".status.status=" 
            + (if .ifOperStatus == "up" then "\"green\"" else "\"blank\"" end)' 
}

# Map the SNMP JSON output to @@STYLUS@@ metadata updates
snmp_check | snmp_parse
```

## Alternatives

For simpler connectivity tests, consider [ping monitoring](../ping/). For server monitoring, consider [SSH monitoring](../ssh/). 