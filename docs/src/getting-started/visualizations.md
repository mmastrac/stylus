# Visualizations

The default **Stylus** webapp provides a few builtin visualizations to get you
up and running quickly.

## Overview

Visualizations are configured in your `config.yaml` file under the `ui.visualizations` section. Each visualization has a type, title, description, and type-specific configuration options.

## Configuration

```yaml
ui:
  visualizations:
    - title: "Service Status"
      description: "Overview of all monitored services"
      type: "table"
    - title: "Network Diagram"
      description: "Visual representation of network topology"
      type: "svg"
      url: "/network.svg"
    - title: "Infrastructure Dashboard"
      description: "Interactive infrastructure overview"
      type: "iframe"
      url: "/dashboard.html"
      inject: true
```

## Visualization Types

### Table Visualization

The table visualization displays monitor status in a structured table format with status indicators and clickable rows for log viewing.

**Configuration:**

```yaml
- title: "Service Status"
  description: "Overview of all monitored services"
  type: "table"
```

### Stack Visualization

The stack visualization displays monitor status like a rack of servers.

**Configuration:**

```yaml
- title: "Rack Status"
  description: "Rack of servers with status indicators"
  type: "stack"
  stacks:
    - title: "Rack 1"
      rows:
        - id: "group"
          size: "small"
          layout: 1x5x2 1x1x2
```

The `size` parameter controls the size of the status indicators, and the `layout`
parameter controls the layout of the status indicators.

### SVG Visualization

The SVG visualization loads an SVG file and applies dynamic styling based on
monitor status. This works well with network diagrams, flowcharts, and other
vector graphics.

The SVG is loaded from the `static/` directory, and is automatically updated
when the status changes.

**Configuration:**

```yaml
- title: "Network Diagram"
  description: "Visual representation of network topology"
  type: "svg"
  url: "/network.svg"
```

The SVG visualization automatically applies your configured CSS rules to the SVG
content. The recommended method is setting data-monitor-id attributes on the SVG
elements, and applying `fill:` CSS rules.

See the [CSS Configuration](../configuration/css/) section for more details.

### Iframe Visualization

The iframe visualization embeds external HTML content with optional style
injection, allowing you to create custom visualizations that fit into the
existing pages. See [Custom Monitor Pages](creating-pages.md) for more details.

**Configuration:**

```yaml
- title: "Infrastructure Dashboard"
  description: "Interactive infrastructure overview"
  type: "iframe"
  url: "/dashboard.html"
  inject: true
```

When `inject: true` is set, the monitor CSS is automatically injected into the
`iframe`, applying to the content within.

### Isoflow Visualization

The Isoflow visualization provides interactive diagrams with dynamic data
updates.

**Configuration:**

```yaml
- title: "Service Flow"
  description: "Interactive service dependency diagram"
  type: "isoflow"
  config: "service-flow"
```

Isoflow visualizations require initial data to be placed in
`config.d/{config-name}.json`. The data is automatically updated with status
information when available.

## Fullscreen Mode

All visualizations support fullscreen mode for detailed viewing. Click the
fullscreen button (`⛶`) in the top-right corner of any visualization card.

## Examples

### Simple Status Dashboard

A table and SVG diagram.

```yaml
ui:
  visualizations:
    - title: "Service Status"
      description: "Overview of all monitored services"
      type: "table"
    - title: "Network Topology"
      description: "Network diagram with status colors"
      type: "svg"
      url: "/network.svg"
```

A table, iframe, and Isoflow diagram.

```yaml
ui:
  visualizations:
    - title: "Service Status"
      description: "Quick overview of all services"
      type: "table"
    - title: "Infrastructure Diagram"
      description: "D3.js infrastructure visualization"
      type: "iframe"
      url: "/infrastructure.html"
      inject: true
    - title: "Service Dependencies"
      description: "Interactive dependency flow"
      type: "isoflow"
      config: "dependencies"
```

## Advanced Usage

### Custom Visualization Development

For complex visualizations, you can create custom HTML/JavaScript applications
and embed them using the `iframe` visualization type. This allows for complex
interactive dashboards, real-time data visualization, and integration with
external monitoring systems.

See the [Custom Monitor Pages](creating-pages.md) section for more details.
