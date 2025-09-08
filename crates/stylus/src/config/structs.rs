use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::monitor::MonitorMessageProcessor;
use crate::monitors::snmp::SnmpNetworkMonitorConfig;

pub enum OperationMode {
    Run(Config, bool),
    Dump(Config),
    Init(PathBuf, bool),
    Test(Config, String),
}

fn default_server_port() -> u16 {
    80
}

fn default_listen_addr() -> String {
    "0.0.0.0".into()
}

pub fn default_server_static() -> PathBuf {
    "static".into()
}

fn default_monitor_dir() -> PathBuf {
    "monitor.d".into()
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: u32,
    pub server: ServerConfig,
    #[serde(default)]
    pub monitor: MonitorConfig,
    #[serde(default)]
    pub css: CssConfig,
    #[serde(default, skip_serializing_if = "default")]
    pub base_path: PathBuf,
    #[serde(default, skip_serializing_if = "default")]
    pub ui: Option<serde_value::Value>,
    #[serde(default, skip_serializing_if = "default")]
    pub config_d: HashMap<String, serde_value::Value>,
}

fn default<T: Default + PartialEq>(t: &T) -> bool {
    *t == Default::default()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    #[serde(default = "default_server_port")]
    pub port: u16,
    #[serde(default = "default_listen_addr")]
    pub listen_addr: String,
    #[serde(default, rename = "static")]
    pub static_path: Option<PathBuf>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_server_port(),
            listen_addr: default_listen_addr(),
            static_path: Some(default_server_static()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonitorConfig {
    #[serde(default = "default_monitor_dir")]
    pub dir: PathBuf,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        Self {
            dir: default_monitor_dir(),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
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
    #[serde(default, skip_serializing_if = "default")]
    pub blank: Arc<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "default")]
    pub red: Arc<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "default")]
    pub yellow: Arc<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "default")]
    pub green: Arc<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "default")]
    pub blue: Arc<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "default")]
    pub orange: Arc<BTreeMap<String, String>>,
}

impl Default for CssMetadataConfig {
    fn default() -> Self {
        Self {
            blank: Arc::new(Default::default()),
            red: Arc::new(Default::default()),
            yellow: Arc::new(Default::default()),
            green: Arc::new(Default::default()),
            blue: Arc::new(Default::default()),
            orange: Arc::new(Default::default()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonitorDirConfig {
    #[serde(flatten)]
    pub root: MonitorDirRootConfig,
    #[serde(default, skip_serializing_if = "default")]
    pub base_path: PathBuf,
    #[serde(default, skip_serializing_if = "default")]
    pub id: String,
}

impl Default for MonitorDirConfig {
    fn default() -> Self {
        Self {
            root: MonitorDirRootConfig::Test(MonitorDirTestConfig::default()),
            base_path: Default::default(),
            id: Default::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum MonitorDirRootConfig {
    Test(MonitorDirTestConfig),
    Group(MonitorDirGroupConfig),
    Snmp(SnmpNetworkMonitorConfig),
}

impl MonitorDirRootConfig {
    /// Get the MonitorDirTestConfig for this.
    pub fn test(&self) -> &MonitorDirTestConfig {
        match self {
            MonitorDirRootConfig::Test(ref test) => test,
            MonitorDirRootConfig::Group(ref group) => &group.test,
            MonitorDirRootConfig::Snmp(ref snmp) => {
                snmp.test.as_ref().expect("test_mut was not called")
            }
        }
    }

    /// Get the MonitorDirTestConfig for this.
    pub fn test_mut(&mut self) -> &mut MonitorDirTestConfig {
        match self {
            MonitorDirRootConfig::Test(ref mut test) => test,
            MonitorDirRootConfig::Group(ref mut group) => &mut group.test,
            MonitorDirRootConfig::Snmp(ref mut snmp) => {
                if snmp.test.is_none() {
                    snmp.test = Some(snmp.test());
                }
                snmp.test.as_mut().unwrap()
            }
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
    #[serde(skip)]
    pub args: Vec<String>,
    #[serde(skip)]
    pub processor: Option<Arc<dyn MonitorMessageProcessor>>,
}
