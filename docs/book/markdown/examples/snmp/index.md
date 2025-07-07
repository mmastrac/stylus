# SNMP Monitoring

SNMP (Simple Network Management Protocol) is a useful way to write more complex checks for network devices, but the output of the tools requires some massaging.

## Basic SNMP Check

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

## When to Use SNMP

SNMP monitoring is ideal for:
- Network device monitoring (switches, routers, etc.)
- Interface status monitoring
- Network performance metrics
- Device health checks
- Legacy network equipment

For simpler connectivity tests, consider [ping monitoring](../ping/). For server monitoring, consider [SSH monitoring](../ssh/). 