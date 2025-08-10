use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use serde::{Deserialize, Serialize};

use crate::{
    config::{MonitorDirAxisValue, MonitorDirChildConfig, MonitorDirTestConfig},
    expressions::{self, Value},
    interpolate::interpolate_id,
    monitor::{MonitorMessageProcessor, MonitorMessageProcessorInstance},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct SnmpNetworkMonitorConfig {
    target: SnmpNetworkMonitorSnmpConfig,
    pub id: String,
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    #[serde(default = "default_include")]
    pub include: String,
    #[serde(default = "default_exclude")]
    pub exclude: String,
    #[serde(default = "default_red")]
    pub red: String,
    #[serde(default = "default_green")]
    pub green: String,
    #[serde(skip_deserializing)]
    pub children: BTreeMap<String, MonitorDirChildConfig>,
    #[serde(skip_deserializing)]
    pub test: Option<MonitorDirTestConfig>,
}

fn default_include() -> String {
    format!("true")
}

fn default_exclude() -> String {
    format!("false")
}

fn default_red() -> String {
    format!("false")
}

fn default_green() -> String {
    format!("ifOperStatus == 'up' and ifAdminStatus == 'up'")
}

impl SnmpNetworkMonitorConfig {
    pub fn test(&self) -> MonitorDirTestConfig {
        let binary = if self.target.bulk {
            "snmpbulkwalk"
        } else {
            "snmpwalk"
        };

        let mut parts: Vec<String> = vec![binary.to_string()];

        // Output formatting: -O sQ
        parts.push("-OsQ".into());

        // Timeout (seconds)
        parts.push("-t".into());
        parts.push(self.timeout.as_secs().to_string());

        // SNMP version
        let version_flag = match self.target.version {
            1 => "1",
            2 => "2c",
            3 => "3",
            _ => "2c",
        };
        parts.push("-v".into());
        parts.push(version_flag.to_string());

        // Auth (v1/v2c vs v3)
        match self.target.version {
            1 | 2 => {
                parts.push("-c".into());
                parts.push(self.target.community.clone());
            }
            3 => {
                if let Some(ref username) = self.target.username {
                    parts.push("-u".into());
                    parts.push(username.into());
                }

                let has_auth =
                    self.target.auth_protocol.is_some() && self.target.auth_password.is_some();
                let has_priv = self.target.privacy_protocol.is_some()
                    && self.target.privacy_password.is_some();

                let level = if has_priv {
                    "authPriv"
                } else if has_auth {
                    "authNoPriv"
                } else {
                    "noAuthNoPriv"
                };
                parts.push("-l".into());
                parts.push(level.to_string());

                if has_auth {
                    parts.push("-a".into());
                    parts.push(self.target.auth_protocol.as_ref().unwrap().to_uppercase());
                    parts.push("-A".into());
                    parts.push({
                        let input: &str = self.target.auth_password.as_ref().unwrap();
                        input.into()
                    });
                }
                if has_priv {
                    parts.push("-x".into());
                    parts.push(
                        self.target
                            .privacy_protocol
                            .as_ref()
                            .unwrap()
                            .to_uppercase(),
                    );
                    parts.push("-X".into());
                    parts.push({
                        let input: &str = self.target.privacy_password.as_ref().unwrap();
                        input.into()
                    });
                }
            }
            _ => {}
        }

        // Agent (host:port)
        if let Some(port) = self.target.port {
            parts.push(format!("{}:{}", self.target.host, port));
        } else {
            parts.push(self.target.host.clone());
        }

        parts.push("ifTable".into());

        MonitorDirTestConfig {
            interval: self.interval,
            timeout: self.timeout,
            command: PathBuf::from("/usr/bin/env"),
            args: parts,
            processor: Some(Arc::new(SnmpMonitorMessageProcessor {
                id: self.id.clone(),
                include: self.include.clone(),
                exclude: self.exclude.clone(),
                red: self.red.clone(),
                green: self.green.clone(),
            })),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct SnmpNetworkMonitorSnmpConfig {
    pub host: String,
    pub port: Option<u16>,
    #[serde(default = "default_version")]
    pub version: u8,
    #[serde(default = "default_community")]
    pub community: String,
    pub username: Option<String>,
    pub auth_protocol: Option<String>,
    pub auth_password: Option<String>,
    pub privacy_protocol: Option<String>,
    pub privacy_password: Option<String>,
    #[serde(default = "default_bulk")]
    pub bulk: bool,
}

fn default_community() -> String {
    "public".to_string()
}

fn default_version() -> u8 {
    2
}

fn default_bulk() -> bool {
    true
}

#[derive(Debug)]
pub struct SnmpMonitorMessageProcessor {
    id: String,
    include: String,
    exclude: String,
    red: String,
    green: String,
}

#[derive(Debug, Default)]
pub struct SnmpMonitorMessageProcessorInstance {
    id: String,
    include: String,
    exclude: String,
    red: String,
    green: String,
    ports: Mutex<HashMap<usize, HashMap<String, Value>>>,
}

impl MonitorMessageProcessor for SnmpMonitorMessageProcessor {
    fn new(&self) -> Box<dyn MonitorMessageProcessorInstance> {
        Box::new(SnmpMonitorMessageProcessorInstance {
            id: self.id.clone(),
            include: self.include.clone(),
            exclude: self.exclude.clone(),
            red: self.red.clone(),
            green: self.green.clone(),
            ports: Default::default(),
        })
    }
}

impl MonitorMessageProcessorInstance for SnmpMonitorMessageProcessorInstance {
    fn process_message(&self, input: &str) -> Vec<String> {
        // Parse the input as <oid>.index = <valud>?
        // Note that value may be missing and input may end in the equals. This is considered an empty string.

        if let Some((left, right)) = input.split_once("=") {
            let left = left.trim();
            let right = right.trim();

            if let Some((oid, index)) = left.rsplit_once(".") {
                let oid = oid.trim();
                if let Ok(index) = index.parse::<usize>() {
                    let v = if let Ok(v) = right.parse::<i64>() {
                        Value::Int(v)
                    } else {
                        Value::Str(right.to_string())
                    };
                    self.ports
                        .lock()
                        .unwrap()
                        .entry(index)
                        .or_default()
                        .insert(oid.to_string(), v);
                    return vec![];
                }
            }
        };

        log::warn!("Unexpected snmpwalk/snmpbulkwalk output: {}", input);
        vec![]
    }

    fn finalize(&self) -> Vec<String> {
        let mut result = vec![];

        for (port_index, port_metadata) in std::mem::take(&mut *self.ports.lock().unwrap()) {
            let include = calculate_bool(&self.include, &port_metadata);
            if !include {
                continue;
            }

            let exclude = calculate_bool(&self.exclude, &port_metadata);
            if exclude {
                continue;
            }

            let red = calculate_bool(&self.red, &port_metadata);
            let green = calculate_bool(&self.green, &port_metadata);

            let mut values = BTreeMap::new();
            values.insert(
                "index".into(),
                MonitorDirAxisValue::Number(port_index as i64),
            );
            let Ok(port_id) = interpolate_id(&values, &self.id) else {
                log::warn!(
                    "Failed to interpolate id for port {}: {:?}",
                    port_index,
                    values
                );
                continue;
            };

            for (oid, value) in port_metadata {
                result.push(format!(
                    "group.{}.status.metadata.{}={:?}",
                    port_id,
                    oid,
                    value.as_str()
                ));
            }

            if red {
                result.push(format!("group.{}.status.status=\"red\"", port_id));
            } else if green {
                result.push(format!("group.{}.status.status=\"green\"", port_id));
            }
        }

        result
    }
}

fn calculate_bool(expression: &str, metadata: &HashMap<String, Value>) -> bool {
    match expressions::expression::calculate(expression, metadata) {
        Ok(Ok(value)) => value.as_bool(),
        Err(e) => {
            log::warn!("Failed to parse expression {:?}: {}", expression, e);
            false
        }
        Ok(Err(e)) => {
            log::warn!("Failed to evaluate expression {:?}: {:?}", expression, e);
            false
        }
    }
}
