# stylus test

Runs the given test immediately and displays the status of the given monitor after it completes.

## Usage

```bash
stylus test [OPTIONS] --monitor <MONITOR> <FILE>
```

## Arguments

- `<FILE>` - The configuration file

## Options

- `-m, --monitor <MONITOR>` - The test to run
- `-v, --verbose...` - Pass multiple times to increase the level of verbosity (overwritten by STYLUS_LOG)
- `-h, --help` - Print help

## Description

The `stylus test` command allows you to run a specific monitor test immediately and see its output without starting the full server.

## Example

```bash
# Test a monitor named "web-server" using config.yaml
stylus test --monitor web-server config.yaml

# Test with verbose output
stylus test -v --monitor web-server config.yaml

# Test using a directory (will look for config.yaml inside)
stylus test --monitor web-server ~/my-stylus/
```

## Output Format

The command outputs three sections:

1. _Monitor Log_: Raw output from the test script execution
2. _State_: JSON representation of the monitor's current state
3. _CSS_: Generated CSS rules for the monitor

```bash session
Monitor Log
-----------

2025-07-09T00:45:40.144844+00:00 [exec  ] Starting
2025-07-09T00:45:40.149592+00:00 [meta  ] status.metadata.rps="RPS: 443"
2025-07-09T00:45:40.149627+00:00 [stdout] Web server is responding normally
2025-07-09T00:45:40.149633+00:00 [stderr] + echo '@@STYLUS@@ status.metadata.rps="RPS: 443"'
2025-07-09T00:45:40.149638+00:00 [stderr] + '[' 5 -lt 8 ']'
2025-07-09T00:45:40.149643+00:00 [stderr] + echo 'Web server is responding normally'
2025-07-09T00:45:40.149646+00:00 [stderr] + exit 0
2025-07-09T00:45:40.149666+00:00 [exec  ] Termination: 0

State
-----

{
  "id": "web-server-1",
  "config": {
    "interval": "3s",
    "timeout": "10s",
    "command": "test.sh"
  },
  "status": {
    "status": "green",
    "code": 0,
    "description": "Success",
    "css": {
      "metadata": {
        "color": "#d0e6a5"
      }
    },
    "metadata": {
      "rps": "RPS: 443"
    },
    "log": [
      "2025-07-09T00:45:40.144844+00:00 [exec  ] Starting",
      "..."
      "2025-07-09T00:45:40.149666+00:00 [exec  ] Termination: 0"
    ]
  },
  "children": {}
}

CSS
---

/* web-server-1 */

/* Default rules */
[data-monitor-id="web-server-1"] {
  --monitor-id: "web-server-1";
  --monitor-status: green;
  --monitor-code: 0;
  --monitor-description: "Success";
  --monitor-metadata-rps: RPS: 443;
}
#web-server-1,
[data-monitor-id="web-server-1"] {
background-color: #d0e6a5 !important;
fill: #d0e6a5 !important;
}

#web-server-1 td:nth-child(2)::after {
content: "status=green retval=0"
}

#web-server-1 td:nth-child(3)::after {
content: "Success"
}
```
