# Server Configuration

The server configuration is stored in `config.yaml` in the root directory of the configuration. This controls the overall server behaviour (including listening ports) and tells **Stylus** where to find your monitors (`monitor.d` by default).

```yaml
# Stylus will fail to load any configuration without a version of 1 (for future extensibility)
version: 1

# HTTP server configuration
server:
  # Listen port (default: 80, but init command uses 8000 for development)
  port: 8000
  # Static file directory
  static: static

# Monitor configuration
monitor:
  # The top-level directory that Stylus looks for monitor directories
  dir: monitor.d

css:
  # Arbitrary metadata can be associated with each of the six states: blank (no state),
  # red (failed), yellow (timed out), green (success), blue (highlight), or orange (warning).

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
    blue:
      color: "#3b82f6"
    orange:
      color: "#f9b356"

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
