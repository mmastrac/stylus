# Isoflow Infrastructure Status Example

This example demonstrates how to use Stylus with a dynamic infrastructure diagram created using D3.js. The diagram shows a typical web application infrastructure with load balancer, web servers, database, and cache components.

## Features

- **Dynamic SVG Diagram**: Uses D3.js to create an interactive infrastructure diagram
- **Real-time Status Updates**: Components change color based on their current status
- **Multiple Component Types**: Demonstrates rectangles, ellipses, and polygons for different infrastructure components
- **Connection Lines**: Shows relationships between components
- **Responsive Design**: Diagram adapts to container size

## Components Monitored

1. **Load Balancer** - Health check every 20 seconds
2. **Web Server** - Response check every 30 seconds  
3. **Database** - Connection check every 45 seconds
4. **Cache** - Service check every 25 seconds

## Running the Example

1. Navigate to this directory:
   ```bash
   cd examples/isoflow
   ```

2. Run Stylus:
   ```bash
   ../../target/release/stylus
   ```

3. Open your browser to `http://localhost:8001`

## How It Works

### Diagram Creation
The diagram is created using D3.js and consists of:
- **SVG Elements**: Each infrastructure component is represented by an SVG shape
- **Monitor IDs**: Each component has a `data-monitor-id` attribute that links it to a monitor
- **CSS Styling**: The `style.css` endpoint dynamically updates colors based on monitor status

### Status Colors
- **Green** (`#d0e6a5`): Component is healthy
- **Yellow** (`#ffdd94`): Component is slow or timing out
- **Red** (`#fa897b`): Component is down or failing

### Test Scripts
Each monitor has a test script that simulates different failure scenarios:
- **Success**: Component responds normally
- **Timeout**: Component is slow to respond
- **Failure**: Component is not responding

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

### Real-world Usage
Replace the test scripts with actual health checks:
- **Web Server**: `curl` to check HTTP responses
- **Database**: Connection tests or query execution
- **Load Balancer**: Health check endpoints
- **Cache**: Redis/Memcached connection tests

## Files Structure

```
isoflow/
├── config.yaml          # Stylus configuration
├── static/
│   └── index.html       # Main status page with diagram
├── monitor.d/
│   ├── web-server/      # Web server monitoring
│   ├── database/        # Database monitoring
│   ├── load-balancer/   # Load balancer monitoring
│   └── cache/          # Cache monitoring
└── README.md           # This file
``` 