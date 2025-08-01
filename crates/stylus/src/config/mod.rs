use std::collections::BTreeMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use clap::Parser;
use itertools::Itertools;
use walkdir::WalkDir;

use self::args::{Args, Commands};
pub use self::structs::*;
use crate::interpolate::*;

mod args;
mod structs;

pub fn parse_config_from_args() -> Result<OperationMode, Box<dyn Error>> {
    let args = Args::parse();

    match args.command {
        Commands::Dump(dump_args) => {
            let path = if let Some(path) = dump_args.directory {
                path
            } else {
                dump_args
                    .force_container_path
                    .expect("No forced container path specified")
            };

            let config = parse_config(&path)?;
            Ok(OperationMode::Dump(config))
        }
        Commands::Test(test_args) => {
            let path = if let Some(path) = test_args.directory {
                path
            } else {
                test_args
                    .force_container_path
                    .expect("No forced container path specified")
            };
            let config = parse_config(&path)?;
            Ok(OperationMode::Test(config, test_args.monitor))
        }
        Commands::Init(init_args) => {
            let docker = init_args.directory.is_none();
            let mut path = if let Some(path) = init_args.directory {
                path
            } else {
                init_args
                    .force_container_path
                    .expect("No forced container path specified")
            };
            let extension = path
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_default();
            match extension.as_str() {
                "yml" | "yaml" => {
                    warn!("Passed configuration location {:?} has a yaml file extension -- inferring directory as parent", path);
                    path = path.parent().expect("No parent directory").into();
                }
                _ => {}
            }
            Ok(OperationMode::Init(path, docker))
        }
        Commands::Run(run_args) => {
            let config_path = if let Some(path) = run_args.config {
                path
            } else {
                let path = run_args
                    .force_container_path
                    .expect("No forced container path specified");
                if !path.exists() {
                    eprintln!("Configuration file {} does not exist.", path.display());
                    eprintln!(
                        "Ensure that you have mounted the configuration folder into the container and have run `init` in the configuration path."
                    );
                    return Err("Configuration file does not exist. Unable to continue.".into());
                }
                path
            };
            let mut config = parse_config(&config_path)?;
            if let Some(port) = run_args.force_container_port {
                config.server.port = port
            };
            if let Some(addr) = run_args.force_container_listen_addr {
                config.server.listen_addr = addr
            }
            debug!("{:?}", config);
            Ok(OperationMode::Run(config, run_args.dry_run))
        }
    }
}

pub fn parse_config(file: &Path) -> Result<Config, Box<dyn Error>> {
    let curr = std::env::current_dir()?;
    let mut path = Path::new(&file).into();
    canonicalize("configuration", Some(&curr), &mut path)?;
    if path.is_file() {
        if path.file_name().unwrap_or_default() == "config.yaml" {
            warn!(
                "Passed configuration location {:?} was a file -- inferring directory from parent",
                file
            );
            path = path.parent().expect("No parent directory").into();
        } else {
            return Err(format!("Either the stylus directory or its config.yaml must be passed as an argument: (got {:?})", file).into());
        }
    }
    let path = path.join("config.yaml");
    if !path.exists() {
        return Err(format!("Configuration file {} does not exist.", path.display()).into());
    }
    let s = std::fs::read_to_string(&path)?;
    parse_config_string(&path, s)
}

/// Given a base path and a relative path, gets the full path (or errors out if it doesn't exist).
fn canonicalize(
    what: &str,
    base_path: Option<&Path>,
    path: &mut PathBuf,
) -> Result<(), Box<dyn Error>> {
    let new = match base_path {
        None => path.clone(),
        Some(base_path) => base_path.join(&path),
    };

    if !new.exists() {
        Err(if let Some(base_path) = base_path {
            format!(
                "{} does not exist ({}, base path was {})",
                what,
                path.to_string_lossy(),
                base_path.to_string_lossy()
            )
        } else {
            format!("{} does not exist ({})", what, path.to_string_lossy())
        }
        .into())
    } else {
        *path = new.canonicalize()?;
        Ok(())
    }
}

pub fn parse_config_string(file: &Path, s: String) -> Result<Config, Box<dyn Error>> {
    let mut config: Config = serde_yaml_ng::from_str(&s)?;
    if Iterator::count(config.base_path.components()) == 0 {
        config.base_path = Path::parent(Path::new(&file))
            .ok_or("Failed to get base path")?
            .into();
    }

    let config_d = config.base_path.join("config.d");
    if config_d.exists() {
        for entry in std::fs::read_dir(config_d)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() && path.extension().unwrap_or_default() == "yaml" {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                let value = serde_yaml_ng::from_str(&std::fs::read_to_string(&path)?)?;
                config.config_d.insert(name, value);
            }
            if path.is_file() && path.extension().unwrap_or_default() == "json" {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                let value = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
                config.config_d.insert(name, value);
            }
        }
    }

    for css in config.css.rules.iter_mut() {
        if css.declarations.contains("monitor.config.id")
            || css.selectors.contains("monitor.config.id")
        {
            let msg = "Found deprecated 'monitor.config.id' in template. Please use 'monitor.id'";
            warn!("{}", msg);
            return Err(msg.into());
        }

        css.declarations = css.declarations.trim().to_string();
        css.selectors = css.selectors.trim().to_string();
    }

    // Canonical paths
    canonicalize("base path", None, &mut config.base_path)?;
    if let Some(static_path) = &mut config.server.static_path {
        canonicalize("static file path", Some(&config.base_path), static_path)?;
    } else {
        let mut static_path = config.base_path.join(default_server_static());
        if canonicalize(
            "static file path",
            Some(&config.base_path),
            &mut static_path,
        )
        .is_ok()
        {
            config.server.static_path = Some(static_path);
        }
    }
    canonicalize(
        "monitor directory path",
        Some(&config.base_path),
        &mut config.monitor.dir,
    )?;

    Ok(config)
}

pub fn parse_monitor_configs(root: &Path) -> Result<Vec<MonitorDirConfig>, Box<dyn Error>> {
    if !root.exists() {
        return Err(format!(
            "Monitor directory {} does not exist",
            root.to_string_lossy()
        )
        .into());
    }

    let mut monitor_configs = vec![];
    for e in WalkDir::new(root)
        .min_depth(1)
        .max_depth(1)
        .follow_links(true)
        .into_iter()
    {
        debug!("Got entry: {e:?}");
        let e = e?;
        if e.file_type().is_dir() {
            let mut p = e.into_path();
            p.push("config.yaml");
            if p.exists() {
                monitor_configs.push(parse_monitor_config(&p)?);
                info!("Found monitor in {:?}", p);
            } else {
                debug!("Ignoring {:?} as there was no config.yaml", p);
            }
        } else {
            debug!("Ignoring {:?} as it was not a directory", e.path());
        }
    }

    if monitor_configs.is_empty() {
        Err(format!(
            "Unable to locate any valid monitor config.yaml files in {}",
            root.to_string_lossy()
        )
        .into())
    } else {
        Ok(monitor_configs)
    }
}

pub fn parse_monitor_config(file: &Path) -> Result<MonitorDirConfig, Box<dyn Error>> {
    let s = std::fs::read_to_string(file)?;
    parse_monitor_config_string(file, s)
}

pub fn parse_monitor_config_string(
    file: &Path,
    s: String,
) -> Result<MonitorDirConfig, Box<dyn Error>> {
    let mut config: MonitorDirConfig = serde_yaml_ng::from_str(&s)?;
    if Iterator::count(config.base_path.components()) == 0 {
        config.base_path = Path::parent(file).ok_or("Failed to get base path")?.into();
    }

    // Canonical paths
    canonicalize("base path", None, &mut config.base_path)?;

    if config.id.is_empty() {
        config.id = file
            .parent()
            .ok_or("Invalid parent")?
            .file_name()
            .ok_or("Invalid file name")?
            .to_string_lossy()
            .to_string();
    }

    let test = config.root.test_mut();
    let executable = config.base_path.join(&test.command);
    if executable.exists() {
        test.command = Path::canonicalize(&executable)?;
    } else {
        let command_line = test.command.to_string_lossy().to_string();
        if !command_line.contains(' ') {
            return Err(format!("Command {} is not available", command_line).into());
        }
        test.args = vec!["-c".to_string(), command_line];
        test.command = PathBuf::from("/bin/sh");
    }

    let mut children = BTreeMap::new();
    if let MonitorDirRootConfig::Group(ref mut group) = config.root {
        for values in group
            .axes
            .iter()
            .map(|axis| axis.values.iter().map(move |v| (v, &axis.name)))
            .multi_cartesian_product()
        {
            let mut axes = BTreeMap::new();
            for val in values {
                axes.insert(val.1.to_owned(), val.0.to_owned());
            }

            let id = interpolate_id(&axes, &group.id)?;
            let child = MonitorDirChildConfig {
                axes,
                test: group.test.clone(),
            };
            children.insert(id, child);
        }
        group.children = children;
    }

    Ok(config)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deserialize_config_test() -> Result<(), Box<dyn Error>> {
        let config = parse_config(Path::new("src/testcases/v1.yaml"))?;
        assert_eq!(config.base_path, Path::new("src/testcases").canonicalize()?);
        Ok(())
    }

    #[test]
    fn deserialize_monitor_test() -> Result<(), Box<dyn Error>> {
        let config = parse_monitor_config_string(
            Path::new("/tmp/test.yaml"),
            r#"
# Explicitly set the id here
id: router
test:
    interval: 60s
    timeout: 30s
    command: /bin/sleep
          "#
            .into(),
        )?;

        match config.root {
            MonitorDirRootConfig::Test(test) => {
                assert_eq!(test.command, Path::new("/bin/sleep").canonicalize()?)
            }
            _ => panic!(""),
        }

        Ok(())
    }

    #[test]
    fn deserialize_monitor_group() -> Result<(), Box<dyn Error>> {
        let config = parse_monitor_config_string(
            Path::new("/tmp/test.yaml"),
            r#"
# Explicitly set the id here
id: router
group:
    id: group-{{ index }}
    axes:
        - values: [1, 2, 3]
          name: index
    test:
        interval: 60s
        timeout: 30s
        command: /bin/sleep
          "#
            .into(),
        )?;

        match config.root {
            MonitorDirRootConfig::Group(group) => {
                assert_eq!(group.test.command, Path::new("/bin/sleep").canonicalize()?)
            }
            _ => panic!(""),
        }

        Ok(())
    }
}
