use std::collections::BTreeMap;
use std::error::Error;
use std::path::{Path, PathBuf};

use itertools::Itertools;
use structopt::StructOpt;

use self::args::Args;
pub use self::structs::*;
use crate::interpolate::*;

mod args;
mod structs;

pub fn parse_config_from_args() -> Result<OperationMode, Box<dyn Error>> {
    let args = Args::from_args();
    let config_path = if let Some(path) = args.force_container_path {
        path
    } else {
        args.config
    };
    let mut config = parse_config(&config_path)?;
    if let Some(port) = args.force_container_port {
        config.server.port = port
    };
    if args.force_container_listen_all {
        panic!("--force-container-listen-all is not yet implemented");
    }
    debug!("{:?}", config);

    if args.dump {
        Ok(OperationMode::Dump(config))
    } else {
        Ok(OperationMode::Run(config))
    }
}

pub fn parse_config(file: &Path) -> Result<Config, Box<dyn Error>> {
    let curr = std::env::current_dir()?;
    let mut path = Path::new(&file).into();
    canonicalize("configuration", Some(&curr), &mut path)?;
    let s = std::fs::read_to_string(&path)?;
    parse_config_string(file, s)
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

    for css in config.css.rules.iter() {
        if css.declarations.contains("monitor.config.id")
            || css.selectors.contains("monitor.config.id")
        {
            warn!("Found deprecated 'monitor.config.id' in template. Please use 'monitor.id'");
            return Err(
                "Found deprecated 'monitor.config.id' in template. Please use 'monitor.id'".into(),
            );
        }
    }

    // Canonical paths
    canonicalize("base path", None, &mut config.base_path)?;
    canonicalize(
        "static file path",
        Some(&config.base_path),
        &mut config.server.r#static,
    )?;
    canonicalize(
        "monitor directory path",
        Some(&config.base_path),
        &mut config.monitor.dir,
    )?;

    Ok(config)
}

pub fn parse_monitor_config(file: &Path) -> Result<MonitorDirConfig, Box<dyn Error>> {
    let s = std::fs::read_to_string(&file)?;
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
    test.command = Path::canonicalize(&config.base_path.join(&test.command))?;

    let mut children = BTreeMap::new();
    if let MonitorDirRootConfig::Group(ref mut group) = config.root {
        for values in group
            .axes
            .iter()
            .map(|axis| axis.values.iter().map(move |v| (v, &axis.name)))
            .multi_cartesian_product()
        {
            let mut vals = BTreeMap::new();
            for val in values {
                vals.insert(val.1, val.0);
            }

            let id = interpolate_id(&vals, &group.id)?;
            children.insert(id, group.test.clone());
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
            &Path::new("/tmp/test.yaml"),
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
            MonitorDirRootConfig::Test(test) => assert_eq!(test.command, Path::new("/bin/sleep")),
            _ => panic!(""),
        }

        Ok(())
    }

    #[test]
    fn deserialize_monitor_group() -> Result<(), Box<dyn Error>> {
        let config = parse_monitor_config_string(
            &Path::new("/tmp/test.yaml"),
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
                assert_eq!(group.test.command, Path::new("/bin/sleep"))
            }
            _ => panic!(""),
        }

        Ok(())
    }
}
