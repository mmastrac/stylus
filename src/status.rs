use crate::config::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub enum StatusState {
    Green,
    Yellow,
    Red,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Status {
    pub config: Config,
    pub monitors: Vec<MonitorState>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorState {
    pub config: MonitorDirConfig,
    pub status: MonitorStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorStatus {
    pub status: StatusState,
    pub metadata: HashMap<String, String>,
    pub code: i32,
    pub description: String,
}
