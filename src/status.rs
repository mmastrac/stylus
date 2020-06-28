use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::config::*;
use crate::interpolate::interpolate_modify;
use crate::worker::LogStream;
use crate::worker::WorkerMessage;

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
    #[serde(skip_serializing_if = "MonitorStatus::is_uninitialized")]
    pub status: MonitorStatus,
    pub log: VecDeque<String>,
    #[serde(skip)]
    pub css: Option<String>,
    pub children: BTreeMap<String, MonitorChildStatus>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitorChildStatus {
    #[serde(skip_serializing_if = "MonitorStatus::is_uninitialized")]
    pub status: MonitorStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitorStatus {
    pub status: Option<StatusState>,
    pub code: i64,
    pub description: String,
    pub css: MonitorCssStatus,
    pub metadata: BTreeMap<String, String>,
    #[serde(skip)]
    pub pending: Option<MonitorPendingStatus>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MonitorPendingStatus {
    pub status: Option<StatusState>,
    pub description: Option<String>,
    pub metadata: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitorCssStatus {
    pub metadata: Arc<BTreeMap<String, String>>,
}

impl MonitorState {
    pub fn process_message(
        &mut self,
        id: &str,
        msg: WorkerMessage,
        config: &CssMetadataConfig,
    ) -> Result<(), Box<dyn Error>> {
        debug!("[{}] Worker message {:?}", id, msg);
        match msg {
            WorkerMessage::Starting => {
                // Note that we don't update the state here
                self.status.pending = None;
                self.log.clear();
            }
            WorkerMessage::LogMessage(stream, m) => {
                let stream = match stream {
                    LogStream::StdOut => "stdout",
                    LogStream::StdErr => "stderr",
                };
                // TODO: Long lines without \n at the end should have some sort of other delimiter inserted
                self.log.push_back(format!("[{}] {}", stream, m.trim_end()));

                // This should be configurable
                while self.log.len() > 100 {
                    self.log.pop_front();
                }
            }
            WorkerMessage::Metadata(expr) => {
                // Make borrow checker happy
                let status = &mut self.status;
                let children = &mut self.children;
                if let Err(err) = interpolate_modify(status, children, &expr) {
                    self.log.push_back(format!("[error ] {}", err));
                    error!("Metadata update error: {}", err);
                } else {
                    self.log.push_back(format!("[meta  ] {}", expr));
                }
            }
            WorkerMessage::AbnormalTermination(s) => {
                self.finish(StatusState::Yellow, -1, s, &config);
            }
            WorkerMessage::Termination(code) => {
                if code == 0 {
                    self.finish(StatusState::Green, code, "Success".into(), &config);
                } else {
                    self.finish(StatusState::Red, code, "Failed".into(), &config);
                }
            }
        }
        Ok(())
    }

    fn finish(
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
            child
                .1
                .status
                .finish(status, code, description.clone(), &config);
        }
    }
}

impl MonitorStatus {
    pub fn new(config: &Config) -> MonitorStatus {
        MonitorStatus {
            status: None,
            code: 0,
            description: "Unknown (initializing)".into(),
            metadata: Default::default(),
            pending: None,
            css: MonitorCssStatus {
                metadata: config.css.metadata.blank.clone(),
            },
        }
    }

    pub fn is_uninitialized(&self) -> bool {
        self.status.is_none()
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
        self.status = Some(status);
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
                self.status = Some(status);
            }
            if let Some(description) = pending_description {
                self.description = description;
            }
        }

        // Update the CSS metadata with the final status
        if let Some(status) = self.status {
            self.css.metadata = match status {
                StatusState::Blank => config.blank.clone(),
                StatusState::Green => config.green.clone(),
                StatusState::Yellow => config.yellow.clone(),
                StatusState::Red => config.red.clone(),
            };
        }
    }
}
