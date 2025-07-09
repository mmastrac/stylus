# Creating Monitors

A monitor is a script that is run periodically to check the status of a system.
Monitors are defined in the `monitor.d` directory in a **Stylus** project.

Monitors consist of a configuration file and a test script. 

Using the project we created in the previous section, let's take a look at the
monitor that was created for us. The configuration from the initial project is:

```bash session
$ cat ~/stylus/monitor.d/monitor/config.yaml
interval: 30s
timeout: 10s
command: test.sh

$ cat ~/stylus/monitor.d/monitor/test.sh
#!/bin/sh
echo 'Write your test script here'
```

The `interval` and `timeout` fields are used to control how often the monitor
is run and how long it is allowed to run for. The `command` field is the path
to the test script.

The test script is a simple shell script that will be run by the monitor.

Additionally to aid in the development of monitors, **Stylus** provides a
`stylus test` command that will run the test script and display the status of
the monitor after it completes.

```bash session
$ stylus test ~/stylus/config.yaml --monitor monitor
Monitor Log
-----------

<timestamp> [exec  ] Starting
<timestamp> [stdout] Write your test script here
<timestamp> [exec  ] Termination: 0

State
-----

{
  "id": "monitor",
  "config": {
    "interval": "30s",
    "timeout": "10s",
    "command": "test.sh"
  },
  "status": {
    "status": "green",
    "code": 0,
    "description": "Success",
    "css": {
      "metadata": {}
    },
    "metadata": {},
    "log": [
      // ...
    ]
  },
  "children": {}
}

CSS
---

/* monitor */

/* Default rules */
[data-monitor-id="monitor"] {
  --monitor-id: "monitor";
  --monitor-status: green;
  --monitor-code: 0;
  --monitor-description: "Success";
}
```

## Modifying the Monitor

Let's say that we want to change the test script to check if the server can see
the internet. We'll using `ping 8.8.8.8` as a proxy test for the internet
existing.

Let's update `test.sh` to:

```bash
#!/bin/sh
ping -c 1 8.8.8.8
```

Now let's run the test again:

```bash session
$ stylus test ~/stylus/config.yaml monitor
Monitor Log
-----------

<timestamp> [exec  ] Starting
<timestamp> [stdout] PING 8.8.8.8 (8.8.8.8): 56 data bytes
<timestamp> [stdout] 64 bytes from 8.8.8.8: icmp_seq=0 ttl=111 time=20.496 ms
<timestamp> [stdout] 
<timestamp> [stdout] --- 8.8.8.8 ping statistics ---
<timestamp> [stdout] 1 packets transmitted, 1 packets received, 0.0% packet loss
<timestamp> [stdout] round-trip min/avg/max/stddev = 20.496/20.496/20.496/0.000 ms
<timestamp> [exec  ] Termination: 0

...
```

As expected, the monitor successfully pings the internet.

## More Complex Monitors

Monitors can be as simple or complex as you need them to be. 

For example, you can use `curl` to request a JSON endpoint from a web service,
and ensure that it returns a healthy response:

```bash
#!/bin/sh
set -xeuf -o pipefail
# Check the health of a service running on the monitor
STATUS=$(curl --fail http://my-web-server:8080/health | jq --raw-output '.status')
if [ "$STATUS" != "OK" ]; then
  echo "Service unhealthy: $STATUS"
  exit 1
fi
```

If you want to monitor network devices, you can often use
[SNMP](https://en.wikipedia.org/wiki/Simple_Network_Management_Protocol) to
extract information from the device. SNMP allows you to query the device for
information about its status using OIDs (Object Identifiers), which are roughly
standardized across different devices.

See the manual for your networking device for the appropriate OIDs to use, or
reference one of the following resources: 

 - <https://ldapwiki.com/wiki/Wiki.jsp?page=SNMP>
 - <https://networklessons.com/cisco/ccie-routing-switching/introduction-to-snmp>

```bash
#!/bin/sh
set -xeuf -o pipefail
# Print the SNMP OID for the system description
snmpwalk -v 2c -c public my-network-router 1.3.6.1.2.1.1.1.0
# Print the SNMP OID for the system uptime
snmpwalk -v 2c -c public my-network-router 1.3.6.1.2.1.1.3.0
```

For more information on complex monitors, see the [examples](../examples/).
