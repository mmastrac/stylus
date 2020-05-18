use std::collections::VecDeque;
use std::error::Error;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;

use walkdir::WalkDir;

use crate::config::*;
use crate::interpolate::interpolate_monitor;
use crate::status::*;
use crate::worker::{monitor_thread, WorkerMessage};

#[derive(Debug)]
struct MonitorThread {
    thread: thread::JoinHandle<()>,
    state: MonitorState,
}

#[derive(Clone)]
pub struct Monitor {
    config: Config,
    monitors: Vec<Arc<Mutex<MonitorThread>>>,
}

impl Monitor {
    pub fn new(config: &Config) -> Result<Monitor, Box<dyn Error>> {
        let config = config.clone();
        let mut monitor_configs = Vec::new();
        for e in WalkDir::new(&config.monitor.dir)
            .min_depth(1)
            .max_depth(1)
            .follow_links(true)
            .into_iter()
        {
            let e = e?;
            if e.file_type().is_dir() {
                let mut p = e.into_path();
                p.push("config.yaml");
                if p.exists() {
                    monitor_configs.push(parse_monitor_config(&p)?);
                    info!("Found monitor in {:?}", p);
                } else {
                    debug!("Ignoring {:?} as there was no config.yaml", p);
                }
            } else {
                debug!("Ignoring {:?} as it was not a directory", e.path());
            }
        }
        let mut monitors = Vec::new();
        for monitor_config in &monitor_configs {
            let monitor_config2 = monitor_config.clone();
            let (tx, rx) = channel();
            let thread = thread::spawn(move || {
                let thread = rx.recv().expect("Unexpected error receiving mutex");
                monitor_thread(monitor_config2, move |id, m| {
                    Self::process_message(id, &thread, m)
                });
            });
            let thread = Arc::new(Mutex::new(MonitorThread {
                thread,
                state: MonitorState {
                    config: monitor_config.clone(),
                    status: MonitorStatus {
                        status: StatusState::Yellow,
                        code: 0,
                        description: "Unknown (initializing)".into(),
                        metadata: Default::default(),
                    },
                    log: Arc::new(Mutex::new(VecDeque::new())),
                },
            }));
            // Let the thread go!
            tx.send(thread.clone())
                .expect("Unexpected error sending mutex");
            monitors.push(thread);
        }
        Ok(Monitor { config, monitors })
    }

    fn process_message(
        id: &String,
        monitor: &Arc<Mutex<MonitorThread>>,
        msg: WorkerMessage,
    ) -> Result<(), Box<dyn Error>> {
        let mut thread = monitor.lock().map_err(|_| "Poisoned mutex")?;
        debug!("[{}] Worker message {:?}", id, msg);
        match msg {
            WorkerMessage::Starting => {
                // Note that we don't update the state here
            }
            WorkerMessage::LogMessage(m) => {
                let mut log = thread.state.log.lock().map_err(|_| "Poisoned mutex")?;
                log.push_back(m);

                // This should be configurable
                while log.len() > 100 {
                    log.pop_front();
                }
            }
            WorkerMessage::AbnormalTermination(s) => {
                thread.state.status.code = -1;
                thread.state.status.description = s;
                thread.state.status.status = StatusState::Yellow;
            }
            WorkerMessage::Termination(code) => {
                thread.state.status.code = code;
                if code == 0 {
                    thread.state.status.description = "Success".into();
                    thread.state.status.status = StatusState::Green;
                } else {
                    thread.state.status.description = "Failed".into();
                    thread.state.status.status = StatusState::Red;
                }
            }
        }
        Ok(())
    }

    pub fn generate_css(&self) -> String {
        let mut css = format!("/* Generated at {:?} */\n", std::time::Instant::now()).to_owned();
        let status = self.status();
        for monitor in status.monitors {
            css += "\n";
            css += &format!("/* {} */\n", monitor.config.id);
            for rule in &self.config.css.rules {
                css += &interpolate_monitor(&monitor, &rule.selectors)
                    .unwrap_or("/* failed */".into());
                css += "{\n";
                css += &interpolate_monitor(&monitor, &rule.declarations)
                    .unwrap_or("/* failed */".into());
                css += "}\n\n";
            }
        }
        css
    }

    pub fn status(&self) -> Status {
        let mut monitors = Vec::new();

        for monitor in &self.monitors {
            let monitor = monitor
                .lock()
                .expect("Failed to lock mutex while updating status");
            let mut state = monitor.state.clone();
            state.status.metadata = match state.status.status {
                StatusState::Green => self.config.css.metadata.green.clone(),
                StatusState::Yellow => self.config.css.metadata.yellow.clone(),
                StatusState::Red => self.config.css.metadata.red.clone(),
            };
            monitors.push(state);
        }

        Status {
            config: self.config.clone(),
            monitors,
        }
    }
}
