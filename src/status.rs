use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::config::*;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all(serialize = "lowercase", deserialize = "lowercase"))]
pub enum StatusState {
    Blank,
    Green,
    Yellow,
    Red,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Status {
    pub config: Config,
    pub monitors: Vec<Arc<Mutex<MonitorState>>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitorState {
    pub id: String,
    pub config: MonitorDirTestConfig,
    pub status: MonitorStatus,
    pub log: VecDeque<String>,
    #[serde(skip)]
    pub css: Option<String>,
    pub children: HashMap<String, MonitorStatus>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitorStatus {
    pub status: StatusState,
    pub code: i64,
    pub description: String,
    pub css: MonitorCssStatus,
    pub metadata: HashMap<String, String>,
    #[serde(skip)]
    pub pending: Option<MonitorPendingStatus>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MonitorPendingStatus {
    pub status: Option<StatusState>,
    pub description: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitorCssStatus {
    pub metadata: Arc<HashMap<String, String>>,
}

impl MonitorState {
    pub fn finish(
        &mut self,
        status: StatusState,
        code: i64,
        description: String,
        config: &CssMetadataConfig,
    ) {
        self.css = None;
        self.status
            .finish(status, code, description.clone(), config);
        for child in self.children.iter_mut() {
            eprintln!("{:?} ", child);
            child.1.finish(status, code, description.clone(), &config);
            eprintln!("{:?} ", child);
        }
    }
}

impl MonitorStatus {
    pub fn new(config: &Config) -> MonitorStatus {
        MonitorStatus {
            status: StatusState::Blank,
            code: 0,
            description: "Unknown (initializing)".into(),
            metadata: Default::default(),
            pending: None,
            css: MonitorCssStatus {
                metadata: config.css.metadata.blank.clone(),
            },
        }
    }

    fn finish(
        &mut self,
        status: StatusState,
        code: i64,
        description: String,
        config: &CssMetadataConfig,
    ) {
        let (pending_status, pending_description, pending_metadata) = self
            .pending
            .take()
            .map(|pending| (pending.status, pending.description, pending.metadata))
            .unwrap_or_default();
        self.code = code;

        // Start with the regular update
        self.status = status;
        self.description = description;
        self.metadata.clear();

        // Metadata can only be updated if the process terminated normally
        if status == StatusState::Green {
            if let Some(metadata) = pending_metadata {
                self.metadata = metadata;
            }
        }

        // Only allow overriding status if it was successful
        if status == StatusState::Green {
            if let Some(status) = pending_status {
                self.status = status;
            }
            if let Some(description) = pending_description {
                self.description = description;
            }
        }

        // Update the CSS metadata with the final status
        self.css.metadata = match self.status {
            StatusState::Blank => config.blank.clone(),
            StatusState::Green => config.green.clone(),
            StatusState::Yellow => config.yellow.clone(),
            StatusState::Red => config.red.clone(),
        };
    }
}
