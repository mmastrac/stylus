use crate::config::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StatusState {
    Green,
    Yellow,
    Red,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Status {
    pub config: Config,
    pub monitors: Vec<MonitorState>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitorState {
    pub config: MonitorDirConfig,
    pub status: MonitorStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitorStatus {
    pub status: StatusState,
    pub metadata: HashMap<String, String>,
    pub code: i32,
    pub description: String,
}
