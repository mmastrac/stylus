use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc::channel;

use walkdir::WalkDir;

use crate::config::*;
use crate::status::*;
use crate::worker::{monitor_thread, WorkerMessage};

#[derive(Debug)]
struct MonitorThread {
    thread: thread::JoinHandle<()>,
    state: MonitorState,
}

#[derive(Debug)]
pub struct Monitor {
    config: Config,
    monitors: Vec<Arc<Mutex<MonitorThread>>>,
}

impl Monitor {
    pub fn new(config: &Config) -> Result<Monitor, Box<dyn Error>> {
        let config = config.clone();
        let mut monitor_configs = Vec::new();
        for e in WalkDir::new(&config.monitor.dir)
            .follow_links(true)
            .into_iter()
        {
            let e = e?;
            if e.file_type().is_dir() {
                let mut p = e.into_path();
                p.push("config.yaml");
                if p.exists() {
                    monitor_configs.push(parse_monitor_config(&p)?)
                }
            }
        }
        let mut monitors = Vec::new();
        for monitor_config in &monitor_configs {
            let monitor_config2 = monitor_config.clone();
            let (tx, rx) = channel();
            let thread = thread::spawn(move || {
                let thread = rx.recv().expect("Unexpected error receiving mutex");
                monitor_thread(monitor_config2, move |m| {
                    Self::process_message(&thread, m)
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
                        metadata: config.css.metadata.yellow.clone(),
                    }
                },
            }));
            // Let the thread go!
            tx.send(thread.clone()).expect("Unexpected error sending mutex");
            monitors.push(thread);
        }
        Ok(Monitor {
            config,
            monitors,
        })
    }

    fn process_message(monitor: &Arc<Mutex<MonitorThread>>, msg: WorkerMessage) -> Result<(), Box<dyn Error>> {
        let mut thread = monitor.lock().map_err(|_| "Poisoned mutex")?;
        match msg {
            WorkerMessage::Starting => {
                // Note that we don't update the state here
            },
            WorkerMessage::LogMessage(..) => {},
            WorkerMessage::AbnormalTermination(s) => {
                thread.state.status.description = s;
                thread.state.status.status = StatusState::Yellow;
            },
            WorkerMessage::Termination(code) => {
                if code == 0 {
                    thread.state.status.description = "Success".into();
                    thread.state.status.status = StatusState::Green;
                } else {
                    thread.state.status.description = "Failed".into();
                    thread.state.status.status = StatusState::Red;
                }
            },
        }
        Ok(())
    }
}
