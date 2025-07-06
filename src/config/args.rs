use clap::{ArgGroup, Parser};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(group = ArgGroup::new("path").required(true))]
pub struct Args {
    // TODO
    // /// Daemonize stylus and detact from the tty
    // #[arg(long, short)]
    // pub daemonize: bool,
    /// Pass multiple times to increase the level of verbosity (overwritten by STYLUS_LOG)
    #[arg(
        name = "verbose",
        long = "verbose",
        short = 'v',
        action = clap::ArgAction::Count,
        global = true
    )]
    verbose: u8,

    /// Dumps the effective configuration without running
    #[arg(long)]
    pub dump: bool,

    /// If specified, runs the given test immediately and displays the status of the given monitor after it completes
    #[arg(long, conflicts_with = "dump")]
    pub test: Option<String>,

    /// The configuration file
    #[arg(name = "FILE", group = "path")]
    pub config: Option<PathBuf>,

    /// Advanced: if running in a container, allows the container to override any port specified in config.yaml
    #[arg(long, env = "FORCE_CONTAINER_PORT")]
    pub force_container_port: Option<u16>,

    /// Advanced: if running in a container, allows the container to specify that stylus should listen on the wildcard address
    #[arg(long, env = "FORCE_CONTAINER_LISTEN_ADDR")]
    pub force_container_listen_addr: Option<String>,

    /// Advanced: if running a container, allows the container to override any path specified on the command line
    #[arg(long, env = "FORCE_CONTAINER_PATH", group = "path")]
    pub force_container_path: Option<PathBuf>,
}
