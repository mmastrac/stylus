# stylus

Stylus is a lightweight status page for home infrastructure. Configure a set of bash scripts that test
the various parts of your infrastructure, set up HTML/SVG with a diagram of your network, and stylus will
generate you a dynamic stylesheet to give you a visual overview of the current state.

## Running

```
brew install deno
deno run --unstable --allow-net --allow-read --allow-run src/main.ts example/config.yaml
```

## Screenshots

!(Screenshot)[docs/screenshot.png]
