There's a few approaches you can take to monitoring. This is not a comprehensive list, but should support most devices you wish to monitor.

* [General tips](#general-tips)
* [HTML/API scraping](#htmlapi-scraping) (via [jq](https://github.com/stedolan/jq) and [pup](https://github.com/ericchiang/pup))
* [Ping](#ping)
* [SSH](#ssh) ([secure shell](https://en.wikipedia.org/wiki/Secure_Shell))
* [SNMP](#snmp) ([simple network management protocol](https://en.wikipedia.org/wiki/Simple_Network_Management_Protocol))

# General tips

## Safe Scripting

Because monitor scripts may have a large number of moving parts, consider using [safe shell scripting](https://sipb.mit.edu/doc/safe-shell/) techniques
to ensure that any failure of any kind will return an error code.

In addition `set -x` can be useful to print all commands that run as part of a monitor script. These are available in the logging endpoints and will
show you the expansion of environment variables.

```bash
#!/bin/bash
set -xeuf -o pipefail
```

## Testing your configurations

As monitor scripts using metadata can be somewhat tricky to get right, *Stylus* includes a `--test` command-line argument to allow you to develop
your test script in a slightly more interactive manner. The output from `--test` will include the test script's stdout and stderr streams, plus the
parsed monitor state as JSON, and the final rendered CSS.

# HTML/API scraping

Some devices can be tested using a simple cURL script:

```bash
curl --silent --max-time 2 <url>
```

In other cases you may want to scrape HTML. The `pup` tool is included in the docker image to make this easier. You can use the `json{}` filter to pass
a pre-processed HTML DOM tree to `jq` for further processing.

This example scrapes the power state from a Web Power Switch 7:

```bash
#!/bin/bash
set -euf -o pipefail

function fetch() {
    curl --silent --max-time 2 --basic -u <credentials> <url> \
        | pup -c "table table tr[bgcolor=#F4F4F4] json{}" \
        | jq "[.[] | [.children | .. | .text? | select(. != null)] | { \"name\": .[1], \"state\": (.[2]==\"ON\") }]"
}

n=0
until [ "$n" -ge 10 ]
do
   HTML=`fetch` && break
   n=$((n+1)) 
done

echo $HTML | \
    jq -r -e ". | to_entries | .[] | \"@@STYLUS@@ group.power-\" + (.key + 1 | tostring) + \".status.status=\" + if .value.state then \"\\\"green\\\"\" else \"\\\"blank\\\"\" end"
```

# Ping

The simplest monitor is a ping script. One ping is usually enough for most cases. You can pass a timeout to ping, but *Stylus* will automatically kill
processes if they run too long.

```bash
#!/bin/bash
ping -c 1 ${STYLUS_MONITOR_ID}
```

# SNMP

SNMP is a useful way to write more complex checks, but the output of the tools requires some massaging. 

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

# SSH

Depending on your security requirements, you may wish to loosen some of your SSH client's security check requirements. By disabling strict host key checking and host IP checking, your monitors will be more reliable but there will be [some tradeoffs](https://security.stackexchange.com/questions/161520/what-is-the-actual-drawback-of-checkhostip-no).

```bash
ssh <...> -oStrictHostKeyChecking=no -oCheckHostIP=no
```

## SSH Configuration

To make your life easier, you can collect all of your SSH credentials in [a configuration file](https://www.ssh.com/ssh/config/). The examples in this section will assume you've got a central SSH configuration file.

```
host pi-*
	User matt
host tower
	User root
host unifi-*
	User admin
host *
	IdentityFile /srv/ssh_id_rsa
```

## Basic SSH check


```bash
# Assumes that `ssh_config` lives in the same folder as this script 
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
SSH_CONFIG=$DIR/ssh_config

ssh_check () {
    local host="$STYLUS_MONITOR_ID"
    ssh -F $SSH_CONFIG $host -oStrictHostKeyChecking=no -oCheckHostIP=no "true"
}

ssh_check
```


## SSH + gather basic hardware info

```bash
# Assumes that `ssh_config` lives in the same folder as this script 
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"
SSH_CONFIG=$DIR/ssh_config

ssh_check () {
    local host="$STYLUS_MONITOR_ID"
    ssh -F $SSH_CONFIG $host -oStrictHostKeyChecking=no -oCheckHostIP=no \
        "uname -a && uptime && cat /proc/cpuinfo | grep -i -E '(hardware|model|stepping|revision)' | sort | uniq"
}

ssh_check
```
