# CSS Configuration

The CSS configuration controls how **Stylus** generates dynamic stylesheets based on monitor status. This includes metadata for different states and CSS rules for styling your HTML/SVG elements.

```yaml
# config.yaml
# version: 1
# server: ...
# monitor: ...

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
    - selectors: |
        #{{monitor.id}},
        [data-monitor-id="{{monitor.id}}"] > *
      declarations: |
        background-color: {{monitor.status.css.metadata.color}} !important;
        fill: {{monitor.status.css.metadata.color}} !important;
    # Add some text for the status/return value of the script
    - selectors: |
        #{{monitor.id}} td:nth-child(2)::after
      declarations: |
        content: "status={{monitor.status.status}} retval={{monitor.status.code}}"
```

## CSS Interpolation

Interpolation is used in the `css` block to control the display. The interpolation library under the hood is [handlebars-rust](https://github.com/sunng87/handlebars-rust) and any of the advanced syntaxes may be used.

Generally a monitor's output is interpolated from its status JSON, which will have a following form like the given example below:

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