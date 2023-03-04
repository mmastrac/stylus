use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::sync::Arc;

use keepcalm::SharedMut;
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

#[derive(Clone, Debug, Serialize)]
pub struct Status {
    pub config: Config,
    pub monitors: Vec<SharedMut<MonitorState>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitorState {
    pub id: String,
    pub config: MonitorDirTestConfig,
    #[serde(skip_serializing_if = "MonitorStatus::is_uninitialized")]
    pub status: MonitorStatus,
    #[serde(skip)]
    pub css: Option<String>,
    pub children: BTreeMap<String, MonitorChildStatus>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MonitorChildStatus {
    pub axes: BTreeMap<String, MonitorDirAxisValue>,

    #[serde(skip_serializing_if = "MonitorStatus::is_uninitialized")]
    pub status: MonitorStatus,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MonitorStatus {
    pub status: Option<StatusState>,
    pub code: i64,
    pub description: String,
    pub css: MonitorCssStatus,
    pub metadata: BTreeMap<String, String>,
    pub log: VecDeque<String>,
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

impl Default for MonitorCssStatus {
    fn default() -> Self {
        Self {
            metadata: Arc::new(Default::default()),
        }
    }
}

impl MonitorState {
    /// Internal use only
    fn new_internal(id: String, config: MonitorDirTestConfig) -> Self {
        MonitorState {
            id,
            config,
            status: Default::default(),
            css: None,
            children: Default::default(),
        }
    }

    fn process_log_message<T: FnMut(&str) -> ()>(
        &mut self,
        stream: &str,
        message: &str,
        direct_logger: &mut T,
    ) {
        let msg = format!("[{}] {}", stream, message);
        direct_logger(&msg);
        self.status.log.push_back(msg);
    }

    pub fn process_message<T: FnMut(&str) -> ()>(
        &mut self,
        id: &str,
        msg: WorkerMessage,
        config: &CssMetadataConfig,
        direct_logger: &mut T,
    ) -> Result<(), Box<dyn Error>> {
        debug!("[{}] Worker message {:?}", id, msg);
        match msg {
            WorkerMessage::Starting => {
                // Note that we don't update the state here
                self.status.pending = None;
                self.status.log.clear();
            }
            WorkerMessage::LogMessage(stream, m) => {
                let stream = match stream {
                    LogStream::StdOut => "stdout",
                    LogStream::StdErr => "stderr",
                };
                // TODO: Long lines without \n at the end should have some sort of other delimiter inserted
                self.process_log_message(stream, m.trim_end(), direct_logger);
            }
            WorkerMessage::Metadata(expr) => {
                // Make borrow checker happy
                let status = &mut self.status;
                let children = &mut self.children;
                if let Err(err) = interpolate_modify(status, children, &expr) {
                    self.process_log_message("error ", &expr, direct_logger);
                    self.process_log_message("error ", &err.to_string(), direct_logger);
                    error!("Metadata update error: {}", err);
                } else {
                    self.process_log_message("meta  ", &expr.to_string(), direct_logger);
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

        for child in self.children.iter_mut() {
            let child_status = &mut child.1.status;
            if child_status.is_pending_status_set() || status != StatusState::Green {
                child_status.finish(status, code, description.clone(), &config);
            } else {
                child_status.finish(StatusState::Blank, code, "".into(), &config);
            }
        }

        self.status.finish(status, code, description, config);
    }
}

impl From<&MonitorDirConfig> for MonitorState {
    fn from(other: &MonitorDirConfig) -> Self {
        let mut state = MonitorState::new_internal(other.id.clone(), other.root.test().clone());
        if let MonitorDirRootConfig::Group(ref group) = other.root {
            for child in group.children.iter() {
                state.children.insert(
                    child.0.clone(),
                    MonitorChildStatus {
                        axes: child.1.axes.clone(),
                        status: MonitorStatus::default(),
                    },
                );
            }
        }
        state
    }
}

impl MonitorStatus {
    pub fn initialize(&mut self, config: &CssMetadataConfig) {
        self.description = "Unknown (initializing)".into();
        self.status = Some(StatusState::Blank);
        self.css.metadata = config.blank.clone();
    }

    pub fn is_pending_status_set(&self) -> bool {
        if let Some(ref pending) = self.pending {
            if pending.status.is_none() {
                return false;
            }
        } else {
            return false;
        }

        true
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

        // Metadata/status can only be overwritten if the process terminated normally
        if status == StatusState::Green {
            if let Some(metadata) = pending_metadata {
                self.metadata = metadata;
            }
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
