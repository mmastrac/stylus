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

## Customization

### Adding New Components

1. Add a new monitor directory in `monitor.d/`
2. Create `config.yaml` and `test.sh` files
3. Add the component to the `components` array in `index.html`
4. Add any connections to the `connections` array

### Modifying the Diagram

The diagram is defined in the JavaScript section of `index.html`. You can:
- Change component positions by modifying `x` and `y` coordinates
- Add new component types by extending the drawing logic
- Modify colors and styling in the CSS section
- Add animations or interactions using D3.js

## Files Structure

```
d3/
├── config.yaml          # Stylus configuration
├── static/
│   └── index.html       # Main status page with diagram
└── monitor.d/           # Monitor scripts
``` 
