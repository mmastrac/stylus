use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

pub enum OperationMode {
    Run(Config),
    Dump(Config),
    Test(Config, String),
}

fn default_server_port() -> u16 {
    80
}

fn default_listen_addr() -> String {
    "0.0.0.0".into()
}

fn default_server_static() -> PathBuf {
    "static".into()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub server: ServerConfig,
    pub monitor: MonitorConfig,
    pub css: CssConfig,
    #[serde(default)]
    pub base_path: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    #[serde(default = "default_server_port")]
    pub port: u16,
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    #[serde(default = "default_server_static")]
    pub r#static: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonitorConfig {
    pub dir: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CssConfig {
    pub metadata: CssMetadataConfig,
    pub rules: Vec<CssRule>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CssRule {
    pub selectors: String,
    pub declarations: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CssMetadataConfig {
    #[serde(default)]
    pub blank: Arc<BTreeMap<String, String>>,
    #[serde(default)]
    pub red: Arc<BTreeMap<String, String>>,
    #[serde(default)]
    pub yellow: Arc<BTreeMap<String, String>>,
    #[serde(default)]
    pub green: Arc<BTreeMap<String, String>>,
}

#[cfg(test)]
impl Default for CssMetadataConfig {
    fn default() -> Self {
        Self {
            blank: Arc::new(Default::default()),
            red: Arc::new(Default::default()),
            yellow: Arc::new(Default::default()),
            green: Arc::new(Default::default()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonitorDirConfig {
    #[serde(flatten)]
    pub root: MonitorDirRootConfig,
    #[serde(default)]
    pub base_path: PathBuf,
    #[serde(default)]
    pub id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum MonitorDirRootConfig {
    Test(MonitorDirTestConfig),
    Group(MonitorDirGroupConfig),
}

impl MonitorDirRootConfig {
    /// Get the MonitorDirTestConfig for this.
    pub fn test(&self) -> &MonitorDirTestConfig {
        match self {
            MonitorDirRootConfig::Test(ref test) => test,
            MonitorDirRootConfig::Group(ref group) => &group.test,
        }
    }

    /// Get the MonitorDirTestConfig for this.
    pub fn test_mut(&mut self) -> &mut MonitorDirTestConfig {
        match self {
            MonitorDirRootConfig::Test(ref mut test) => test,
            MonitorDirRootConfig::Group(ref mut group) => &mut group.test,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonitorDirGroupConfig {
    pub id: String,
    pub test: MonitorDirTestConfig,
    pub axes: Vec<MonitorDirAxisConfig>,
    #[serde(skip_deserializing)]
    pub children: BTreeMap<String, MonitorDirChildConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonitorDirChildConfig {
    pub axes: BTreeMap<String, MonitorDirAxisValue>,
    pub test: MonitorDirTestConfig,
}

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
pub enum MonitorDirAxisValue {
    String(String),
    Number(i64),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonitorDirAxisConfig {
    pub values: Vec<MonitorDirAxisValue>,
    pub name: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonitorDirTestConfig {
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    pub command: PathBuf,
}
