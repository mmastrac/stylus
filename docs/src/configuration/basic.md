The *Stylus* configuration is split into two categories:

## Server Configuration

The first is the main server configuration, stored in `config.yaml` in the root directory of the configuration. This controls the overall server behaviour (including listening ports) and the CSS layout. It also points *Stylus* to the monitoring directory (`monitor.d` by default).

```yaml
# Stylus will fail to load any configuration without a version of 1 (for future extensibility)
version: 1

# HTTP server configuration
server:
  # Listen port
  port: 8000
  # Static file directory
  static: static

# Monitor configuration
monitor:
  # The top-level directory that stylus looks for monitor directories
  dir: monitor.d

css:
  # Arbitrary metadata can be associated with each of the four states: blank (no state),
  # red (failed), yellow (timed out) or green (success).

  # Use metadata to get prettier colors - note that we can add arbitrary string keys and values here
  metadata:
    blank:
      color: "white"
    red:
      color: "#fa897b"
    yellow:
      color: "#ffdd94"
    green:
      color: "#d0e6a5"

  # Specify a number of rules - selector/declaration pairs. Each pair will generate a CSS block.
  rules:
    # Style the HTML/SVG with the appropriate status color
    - selectors: "
        #{{monitor.id}},
        [data-monitor-id=\"{{monitor.id}}\"] > *
      "
      declarations: "
        background-color: {{monitor.status.css.metadata.color}} !important;
        fill: {{monitor.status.css.metadata.color}} !important;
      "
    # Add some text for the status/return value of the script
    - selectors: "
        #{{monitor.id}} td:nth-child(2)::after
      "
      declarations: "
        content: \"status={{monitor.status.status}} retval={{monitor.status.code}}\"
      "

```

## Monitor Configuration

### Standard Monitor

A standard monitor consists of a single test for a single host.

```yaml
test:
  # (optional) The internal ID to use for this test. If omitted, the ID is inferred from the monitor directory's name.
  id: foo
  # How often the test is run. The interval restarts from the last success or failure of the test.
  interval: 60s
  # How long the script will be given to run before it is killed.
  timeout: 30s
  # The test command to run, relative to the monitor directory. The PATH is not used and the file must be
  # directly executable.
  command: test.sh
```

Output from the test's standard output and standard error streams are captured and available from the logging endpoint.
A test that has not run or completed yet will be in a blank state. A test that has timed out will be in the yellow state.
Tests that fail by returning a value other than zero will be red, and those that return zero will be in the green state.

Tests scripts may also set metadata associated with the run. More information on this is available in [[Advanced Configuration]].

### Group Monitor

A group may be configured such that a single script may update states for multiple monitors. See [[Advanced Configuration]] for examples
of configuring such a group monitor.

## CSS Interpolation

Interpolation is used in the `css` block to control the display. The interpolation library under the hood is
[handlebars-rust](https://github.com/sunng87/handlebars-rust) and any of the advanced syntaxes may be used.

Generally a monitor's output is interpolated from its status JSON, which will have a following form like the given
example below:

```json
{
  "id": "my-id",
  "config": {
    "interval": "1m",
    "timeout": "30s",
    "command": "/full/path/to/test.sh"
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
      "key": "value1"
    }
  }
}
```

The root object is named `monitor`, and you may choose to use any of the keys as such:

```
{{monitor.id}} = my-id
{{monitor.status.status}} = green
{{monitor.status.css.metadata.color}} = #d0e6a5
{{monitor.status.metadata.key}} = value1
```

You may use additional text content around the interpolation blocks. For example, `background-color: {{monitor.status.css.metadata.color}} !important;` will interpolate to `background-color: #d0e6a5 !important`.

