# stylus

Stylus is a lightweight status page for home infrastructure. Configure a set of bash scripts that test
the various parts of your infrastructure, set up HTML/SVG with a diagram of your network, and stylus will
generate you a dynamic stylesheet to give you a visual overview of the current state.

## Running

```
brew install deno
deno run --unstable --allow-net --allow-read --allow-run src/main.ts example/config.yaml
```

## Configuration

Example configuration for a status page. This configuration attaches metadata to the various states and has
selectors that apply to both and HTML (for a status table) and CSS (for a status SVG image).

```
version: 1
server:
  port: 8000
  static: static

monitor:
  dir: monitor.d

css:
  metadata:
    red:
      color: "#fa897b"
    yellow:
      color: "#ffdd94"
    green:
      color: "#d0e6a5"
  rules:
    - selectors: "
        #${monitor.config.id},
        [data-monitor-id=\"${monitor.config.id}\"] > *
      "
      declarations: "
        background-color: ${monitor.status.metadata.color} !important;
        fill: ${monitor.status.metadata.color} !important;
      "
    - selectors: "
        #${monitor.config.id} td:nth-child(2)::after
      "
      declarations: "
        content: \"status=${monitor.status.status} retval=${monitor.status.code}\"
      "
    - selectors: "
        #${monitor.config.id} td:nth-child(3)::after
      "
      declarations: "
        content: \"${monitor.status.description}\"
      "
```

## Screenshots

![Screenshot](docs/screenshot.png)
