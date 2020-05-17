use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::Duration;

fn default_server_port() -> u16 {
    80
}

fn default_server_static() -> PathBuf {
    PathBuf::from("static")
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub version: u32,
    pub server: ServerConfig,
    pub monitor: MonitorConfig,
    pub css: CssConfig,
    #[serde(default)]
    pub base_path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_port")]
    pub port: u16,
    #[serde(default = "default_server_static")]
    pub r#static: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitorConfig {
    pub dir: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CssConfig {
    pub metadata: MetadataConfig,
    pub rules: Vec<CssRule>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CssRule {
    pub selectors: String,
    pub declarations: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetadataConfig {
    pub red: HashMap<String, String>,
    pub yellow: HashMap<String, String>,
    pub green: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitorDirConfig {
    pub test: MonitorDirTestConfig,
    #[serde(default)]
    pub base_path: PathBuf,
    #[serde(default)]
    pub id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitorDirTestConfig {
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    pub script: PathBuf,
}

pub fn parse_config(file: String) -> Result<Config, Box<dyn Error>> {
    let mut config: Config = serde_yaml::from_str(&std::fs::read_to_string(&file)?)?;
    if Iterator::count(config.base_path.components()) == 0 {
        config.base_path = Path::parent(Path::new(&file))
            .ok_or("Failed to get base path")?
            .into();
    }

    // Canonical paths
    config.base_path = Path::canonicalize(&config.base_path)?.into();
    config.server.r#static =
        Path::canonicalize(&config.base_path.join(&config.server.r#static))?.into();
    config.monitor.dir = Path::canonicalize(&config.base_path.join(&config.monitor.dir))?.into();

    // Basic checks before we return the config
    if !config.server.r#static.exists() {
        Err("Static directory does not exist".into())
    } else if !config.monitor.dir.exists() {
        Err("Monitor directory does not exist".into())
    } else {
        Ok(config)
    }
}

pub fn parse_monitor_config(file: &Path) -> Result<MonitorDirConfig, Box<dyn Error>> {
    let mut config: MonitorDirConfig = serde_yaml::from_str(&std::fs::read_to_string(&file)?)?;
    if Iterator::count(config.base_path.components()) == 0 {
        config.base_path = Path::parent(Path::new(&file))
            .ok_or("Failed to get base path")?
            .into();
    }

    // Canonical paths
    config.base_path = Path::canonicalize(&config.base_path)?.into();
    config.test.script = Path::canonicalize(&config.base_path.join(&config.test.script))?.into();

    if config.id.is_empty() {
        config.id = file
            .parent()
            .ok_or("Invalid parent")?
            .file_name()
            .ok_or("Invalid file name")?
            .to_string_lossy()
            .to_string();
    }

    // Basic checks before we return the config
    if !config.test.script.exists() {
        Err("Test script does not exist".into())
    } else {
        Ok(config)
    }
}
