use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    // TODO
    // /// Daemonize stylus and detact from the tty
    // #[structopt(long, short, parse(from_flag))]
    // pub daemonize: bool,
    /// Pass multiple times to increase the level of verbosity (overwritten by STYLUS_LOG)
    #[structopt(
        name = "verbose",
        long = "verbose",
        short = "v",
        parse(from_occurrences),
        global = true
    )]
    verbose: u8,

    /// Dumps the effective configuration without running
    #[structopt(long, parse(from_flag))]
    pub dump: bool,

    /// If specified, runs the given test immediately and displays the status of the given monitor after it completes
    #[structopt(long, conflicts_with = "dump")]
    pub test: Option<String>,

    /// The configuration file
    #[structopt(name = "FILE", parse(from_os_str), group = "path")]
    pub config: Option<PathBuf>,

    /// Advanced: if running in a container, allows the container to override any port specified in config.yaml
    #[structopt(long, env = "FORCE_CONTAINER_PORT")]
    pub force_container_port: Option<u16>,

    /// Advanced: if running in a container, allows the container to specify that stylus should listen on the wildcard address
    #[structopt(long, env = "FORCE_CONTAINER_LISTEN_ADDR")]
    pub force_container_listen_addr: Option<String>,

    /// Advanced: if running a container, allows the container to override any path specified on the command line
    #[structopt(long, env = "FORCE_CONTAINER_PATH", group = "path")]
    pub force_container_path: Option<PathBuf>,
}
