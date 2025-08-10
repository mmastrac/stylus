# stylus [![CI](https://github.com/mmastrac/stylus/actions/workflows/build.yml/badge.svg)](https://github.com/mmastrac/stylus/actions/workflows/build.yml) [![crates.io](https://img.shields.io/crates/v/stylus.svg)](https://crates.io/crates/stylus) [![Docker Pulls](https://img.shields.io/docker/pulls/mmastrac/stylus.svg)](https://hub.docker.com/r/mmastrac/stylus) [![Book](https://img.shields.io/badge/book-online-blue)](https://mmastrac.github.io/stylus/)

**Stylus** (_style + status_) is a lightweight status page for infrastructure
and networks. Configure a set of bash scripts that test the various parts of
your infrastructure, set up visualizations with minimal configuration, and
**Stylus** will generate you a dashboard for your system.

![Screenshot](docs/src/screenshots/screenshot-1.png)

## Running

**Stylus** is easy to install and run. Docker images are available for the most
common platforms.

```bash
mkdir ~/stylus
docker run --rm --name stylus -p 8000:8000 -v ~/stylus/:/srv mmastrac/stylus:latest init
docker run --rm --name stylus -p 8000:8000 -v ~/stylus/:/srv mmastrac/stylus:latest
```

You can also run **Stylus** without Docker by installing the `stylus` binary
from crates.io.

```bash
cargo install stylus
stylus init ~/stylus
stylus run ~/stylus
```

For more information, [see the book page on running Stylus here](https://mmastrac.github.io/stylus/getting-started/running.html).

## Configuration

Example `config.yaml` for a **Stylus** install. This configuration attaches
metadata to the various states and has selectors that apply to both and HTML
(for a status table) and CSS (for a status SVG image).

```yaml
version: 1
server:
  port: 8000
  static: static/

monitor:
  dir: monitor.d/

ui:
  title: Stylus Monitor
  description: Real-time monitoring of your services
  visualizations:
    - title: Monitor List
      description: List of all monitors in table view
      type: table
```

The monitors are configured by creating a subdirectory in the monitor directory
(default `monitor.d/`) and placing a `config.yaml` in that monitor subdirectory.

```yaml
# ID is optional and will be inferred from the directory
id: router-1
test:
  interval: 60s
  timeout: 30s
  command: test.sh
```

## Test scripts

The test scripts are usually pretty simple. Note that the docker container ships
with a number of useful utilities, but you can consider manually installing
additional packages (either creating an additional docker container or manually
running alpine's `apk` tool inside the container) to handle your specific cases.

### Ping

Unless you have a particularly lossy connection, a single ping should be enough
to test whether a host is up:

```bash
#!/bin/bash
set -xeuf -o pipefail
ping -c 1 8.8.8.8
```

### cURL

For hosts with services that may be up or down, you may want to use cURL to test
whether the service itself is reachable.

```bash
#!/bin/bash
set -xeuf -o pipefail
curl --retry 2 --max-time 5 --connect-timeout 5 http://192.168.1.1:9000
```

### SNMP

**Stylus** has a built-in SNMP monitor that can be used to monitor network
devices. 

```yaml
snmp:
  id: router-{{ index }}
  interval: 60s
  timeout: 30s
  exclude: |
    ifType != 'ethernetCsmacd'
  red: |
    ifOperStatus == "up" and ifSpeed < 1000000000
  target:
    host: 192.168.1.254
    community: public
```

### Advanced techniques

Tools such as `jq`, `sed`, or `awk` can be used for more advanced tests (ie:
APIs). If needed, ssh can be used to connect to hosts and remote tests can be
executed. `snmpwalk` and `snmpget` can also be used to construct tests for
devices that speak SNMP.

If you have an existing **grafana** instance, you can use that as a monitoring
source. See the [Grafana HTTP
API](https://grafana.com/docs/grafana/latest/developers/http_api/) documentation
for more information.

## Performance

**Stylus** is very lightweight, both from a processing and memory perspective.

On a Raspberry Pi 1B, **Stylus** uses less than 1% of CPU while refreshing CSS
at a rate of 1/s. On a 2015 MacBook Pro, Stylus uses approximately 0.1% of a
single core while actively refreshing.

**Stylus** uses approxmately 2MB to monitor 15 services on a Raspberry Pi
(according to
[ps_mem](https://raw.githubusercontent.com/pixelb/ps_mem/master/ps_mem.py)).

When not actively monitored, **Stylus** uses a nearly unmeasurable amount of CPU
and is pretty much limited by how heavyweight your test scripts are.

## More Screenshots

### D3.js example

![Screenshot](docs/src/screenshots/screenshot-3.png)

### A basic home network diagram

![Screenshot](docs/src/screenshots/screenshot-2.png)

## Historical Note

Note that this project was originally written using deno, but was rewritten in
Rust to support Raspberry Pis. The original deno source is available in the
`deno` branch.
