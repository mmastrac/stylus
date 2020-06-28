use std::error::Error;
use std::sync::mpsc::{channel, TryRecvError};
use std::sync::{Arc, Mutex};
use std::thread;

use crate::config::*;
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
    pub config: Config,
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
                thread.lock().expect("Poisoned mutex").process_message(
                    id,
                    m,
                    &css_config,
                    &mut |_| {},
                )
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
    pub fn new(config: &Config) -> Result<Monitor, Box<dyn Error>> {
        let config = config.clone();
        let mut monitors = Vec::new();
        for monitor_config in &parse_monitor_configs(&config.monitor.dir)? {
            monitors.push(MonitorThread::create(
                monitor_config.clone(),
                monitor_config.into(),
                config.css.metadata.clone(),
            )?);
        }
        Ok(Monitor { config, monitors })
    }

    pub fn status(&self) -> Status {
        Status {
            config: self.config.clone(),
            monitors: self.monitors.iter().map(|m| m.state.clone()).collect(),
        }
    }
}
