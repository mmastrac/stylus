use std::collections::BTreeMap;
use std::error::Error;
use std::sync::mpsc::{channel, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

use walkdir::WalkDir;

use crate::config::*;
use crate::interpolate::*;
use crate::status::*;
use crate::worker::{monitor_thread, ShuttingDown};

/// We don't want to store the actual sender in the MonitorThread, just a boxed version of it that
/// will correctly drop to trigger the thread to shut down.
trait OpaqueSender: std::fmt::Debug + Send + Sync {}

impl<T> OpaqueSender for T where T: std::fmt::Debug + Send + Sync {}

#[derive(Debug)]
struct MonitorThread {
    sender: Option<Arc<dyn OpaqueSender>>,
    thread: Option<thread::JoinHandle<()>>,
    state: Arc<Mutex<MonitorState>>,
}

#[derive(Debug)]
pub struct Monitor {
    config: Config,
    monitors: Vec<MonitorThread>,
}

impl MonitorThread {
    /// Create a new monitor thread and release it
    fn create(
        monitor: MonitorDirConfig,
        mut state: MonitorState,
        css_config: CssMetadataConfig,
    ) -> Result<Self, Box<dyn Error>> {
        let (tx, rx) = channel();
        state.status.initialize(&css_config);
        for state in &mut state.children {
            state.1.status.initialize(&css_config);
        }
        let state = Arc::new(Mutex::new(state));

        let thread = thread::spawn(move || {
            let thread: Arc<Mutex<MonitorState>> =
                rx.recv().expect("Unexpected error receiving mutex");
            monitor_thread(monitor, move |id, m| {
                if let Err(TryRecvError::Disconnected) = rx.try_recv() {
                    return Err(ShuttingDown::default().into());
                }
                thread
                    .lock()
                    .expect("Poisoned mutex")
                    .process_message(id, m, &css_config)
            });
        });

        // Let the thread go!
        tx.send(state.clone())
            .expect("Unexpected error sending mutex");

        let thread = MonitorThread {
            thread: Some(thread),
            state,
            sender: Some(Arc::new(Mutex::new(tx))),
        };

        Ok(thread)
    }
}

impl Drop for MonitorThread {
    fn drop(&mut self) {
        // Close the channel
        self.sender.take();

        // Note that we don't try to join the thread here as there's no way to timeout
        self.thread.take();
    }
}

impl Monitor {
    pub fn new(config: &Config, start: bool) -> Result<Monitor, Box<dyn Error>> {
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
            let mut state =
                Self::create_state(monitor_config.id.clone(), &monitor_config.root.test());
            if let MonitorDirRootConfig::Group(ref group) = monitor_config.root {
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
            if start {
                monitors.push(MonitorThread::create(
                    monitor_config.clone(),
                    state,
                    config.css.metadata.clone(),
                )?);
            } else {
                // If we're not starting the monitors just push the state. This is not a great solution
                // to deal with the "dump" mode.
                monitors.push(MonitorThread {
                    sender: None,
                    thread: None,
                    state: Arc::new(Mutex::new(state)),
                })
            }
        }
        Ok(Monitor { config, monitors })
    }

    fn create_state(id: String, monitor_config: &MonitorDirTestConfig) -> MonitorState {
        MonitorState {
            id,
            config: monitor_config.clone(),
            status: MonitorStatus::default(),
            css: None,
            children: BTreeMap::new(),
        }
    }

    pub fn generate_css(&self) -> String {
        let mut css = format!("/* Generated at {:?} */\n", std::time::Instant::now());
        let status = self.status();
        for monitor in status.monitors {
            css += "\n";
            let mut monitor = monitor.lock().expect("Poisoned mutex");

            // Build the css from cache
            let mut cache = monitor.css.take();
            css +=
                cache.get_or_insert_with(|| self.generate_css_for_monitor(&monitor.id, &monitor));
            monitor.css = cache;
        }
        css
    }

    fn generate_css_for_monitor(&self, id: &str, monitor: &MonitorState) -> String {
        let mut css = format!("/* {} */\n", id);
        for rule in &self.config.css.rules {
            css += &interpolate_monitor(id, &monitor.config, &monitor.status, &rule.selectors)
                .unwrap_or_else(|_| "/* failed */".into());
            css += "{\n";
            css += &interpolate_monitor(id, &monitor.config, &monitor.status, &rule.declarations)
                .unwrap_or_else(|_| "/* failed */".into());
            css += "}\n\n";
        }
        for child in monitor.children.iter() {
            for rule in &self.config.css.rules {
                css += &interpolate_monitor(
                    child.0,
                    &monitor.config,
                    &child.1.status,
                    &rule.selectors,
                )
                .unwrap_or_else(|_| "/* failed */".into());
                css += "{\n";
                css += &interpolate_monitor(
                    child.0,
                    &monitor.config,
                    &child.1.status,
                    &rule.declarations,
                )
                .unwrap_or_else(|_| "/* failed */".into());
                css += "}\n\n";
            }
        }
        css
    }

    pub fn status(&self) -> Status {
        Status {
            config: self.config.clone(),
            monitors: self.monitors.iter().map(|m| m.state.clone()).collect(),
        }
    }
}
