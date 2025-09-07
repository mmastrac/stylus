# Row Visualization Example

This example demonstrates the row visualization layout in Stylus, mixing a table component and an iframe component side by side.

## Features

- **Row Layout**: Uses the `row` visualization type to create a horizontal layout
- **Mixed Components**: Combines a table (width: 2) and iframe (width: 3) in a 2:3 ratio
- **Real-time Dashboard**: Custom HTML dashboard with live status updates
- **Monitor Integration**: Uses the same monitors as the simple_network example

## Components

### Left Side - Table (width: 2)
- Standard table visualization showing monitor status
- Displays name, status, and other details in tabular format

### Right Side - Dashboard (width: 3)
- Custom HTML dashboard (`static/dashboard.html`)
- Visual status indicators styled via CSS injection
- Network metrics summary with real-time updates
- Automatic status color updates using `data-monitor-id` attributes

## Monitors

The example includes three monitors copied from the simple_network example:

1. **router** - Pings Google's DNS (8.8.8.8) to test internet connectivity
2. **server** - Attempts to connect to a local service (192.168.1.1:9000)
3. **timeout** - Intentionally times out to demonstrate failure states

## Running the Example

```bash
cd examples/row
stylus
```

Then visit http://localhost:8000 to see the row visualization in action.

## Configuration

The layout is configured in `config.yaml`:

```yaml
visualizations:
  - title: "Network Dashboard"
    type: "row"
    columns:
      - type: "table"
        width: 2
      - type: "iframe"
        width: 3
        url: "/dashboard.html"
        inject: true
```

The `inject: true` setting enables CSS injection into the iframe, automatically styling elements with `data-monitor-id` attributes based on monitor status. This allows the dashboard to visually update in real-time without custom JavaScript for status handling.
