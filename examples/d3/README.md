# Isoflow Infrastructure Status Example

This example demonstrates how to use Stylus with a dynamic infrastructure diagram created using D3.js. The diagram shows a typical web application infrastructure with load balancer, web servers, database, and cache components.

## Running the Example

```bash
cargo run -- examples/d3/
```

3. Open your browser to `http://localhost:8000`

## How It Works

### Diagram Creation

The diagram is created using D3.js and creates a number of SVG elements with a
`data-monitor-id` attribute that links it to a monitor. The `style.css` endpoint
dynamically updates colors based on monitor status:

```yaml
    # Style the HTML/SVG with the appropriate status color
    - selectors: "
        #{{monitor.id}},
        [data-monitor-id=\"{{monitor.id}}\"]
      "
      declarations: "
        background-color: {{monitor.status.css.metadata.color}} !important;
        fill: {{monitor.status.css.metadata.color}} !important;
      "
```

## Files Structure

```
d3/
├── config.yaml          # Stylus configuration
├── static/
│   └── index.html       # Main status page with diagram
└── monitor.d/           # Monitor scripts
``` 

## Caveats

This example uses CDN resources for external libraries. This is not recommended for production.
