use std::collections::BTreeMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use clap::Parser;
use itertools::Itertools;
use walkdir::WalkDir;

use self::args::Args;
pub use self::structs::*;
use crate::interpolate::*;

mod args;
mod structs;

pub fn parse_config_from_args() -> Result<OperationMode, Box<dyn Error>> {
    let args = Args::parse();
    let config_path = if let Some(path) = args.force_container_path {
        path
    } else {
        args.config.unwrap()
    };
    let mut config = parse_config(&config_path)?;
    if let Some(port) = args.force_container_port {
        config.server.port = port
    };
    if let Some(addr) = args.force_container_listen_addr {
        config.server.listen_addr = addr
    }
    debug!("{:?}", config);

    if args.dump {
        Ok(OperationMode::Dump(config))
    } else if let Some(test) = args.test {
        Ok(OperationMode::Test(config, test))
    } else {
        Ok(OperationMode::Run(config))
    }
}

pub fn parse_config(file: &Path) -> Result<Config, Box<dyn Error>> {
    let curr = std::env::current_dir()?;
    let mut path = Path::new(&file).into();
    canonicalize("configuration", Some(&curr), &mut path)?;
    if path.is_dir() {
        warn!("Passed configuration location {:?} was a directory -- inferring 'config.yaml' in that directory", file);
        path = path.join("config.yaml");
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
    let mut config: Config = serde_yaml::from_str(&s)?;
    if Iterator::count(config.base_path.components()) == 0 {
        config.base_path = Path::parent(Path::new(&file))
            .ok_or("Failed to get base path")?
            .into();
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
    let mut config: MonitorDirConfig = serde_yaml::from_str(&s)?;
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
        test.args = vec!["-c".to_string(), test.command.to_string_lossy().to_string()];
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
