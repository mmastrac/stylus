# Running Stylus

You can run **Stylus** in several ways:

* [Using Docker](#docker)
* [Using Static Binaries](#static-binaries) 
* [Using Cargo](#cargo)
* [Building from Source](#from-source)

### Docker

The recommended option is to use Docker. A multi-arch Docker container is
available under the repository `mmastrac/stylus` at
<https://hub.docker.com/r/mmastrac/stylus/>.

Note that the container is hard-wired to run against a configuration file
located in the container at `/srv/config.yaml`, and assumes that the remainder
of the configuration is located in subdirectories of `/srv`. 

You should map the container's `/srv` directory to a local directory containing
your configuration.

```bash
# Assume that this is running against the stylus example, this will map the example directory into
# the container's /srv folder. The container will automatically load config.yaml from this folder!
docker run --name stylus -p 8000:8000 -v ~/stylus/:/srv mmastrac/stylus:latest
```

### Static Binaries

If you would like to run it from a static binary, you may find a number of
pre-built binary releases at <https://github.com/mmastrac/stylus/releases>.

```bash
# This will run against the example in ~/stylus/
stylus_<arch> ~/stylus/config.yaml
```

### Cargo

For any platform where `cargo` is natively available, you can simply `cargo
install` the `stylus` package.

```bash
cargo install stylus
stylus ~/stylus/config.yaml
```

### From Source

If you have the source downloaded, you can run `stylus` directly from that source directory.

```bash
cargo run -- ~/stylus/config.yaml
```
