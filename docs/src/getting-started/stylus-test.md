# stylus test

Runs the given test immediately and displays the status of the given monitor after it completes.

## Usage

```bash
stylus test [OPTIONS] --monitor <MONITOR> <FILE>
```

## Arguments

- `<DIRECTORY>` - The configuration directory

## Options

- `-m, --monitor <MONITOR>` - The test to run
- `-v, --verbose...` - Pass multiple times to increase the level of verbosity (overwritten by STYLUS_LOG)
- `-h, --help` - Print help

The `stylus test` command runs a specific monitor test immediately and shows you the output without starting the full server. This is perfect for debugging monitor scripts or checking if your configuration works.

## Example

```bash
# Test a monitor named "web-server"
stylus test --monitor web-server ~/my-stylus/

# Test with verbose output
stylus test -v --monitor web-server ~/my-stylus/
```

The command shows you three things:

1. _Monitor Log_: What your script actually output
2. _State_: How **Stylus** interpreted the results
3. _CSS_: The CSS rules that would be generated

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
