use std::error::Error;
use std::thread;

use keepcalm::SharedMut;

use crate::config::*;
use crate::status::*;
use crate::worker::{monitor_thread, ShuttingDown};

#[derive(Debug)]
struct MonitorThread {
    /// This is solely used to detect when [`MonitorThread`] is dropped.
    #[allow(unused)]
    drop_detect: SharedMut<()>,
    state: SharedMut<MonitorState>,
}

#[derive(Debug)]
pub struct Monitor {
    pub config: Config,
    monitors: Vec<MonitorThread>,
}

pub trait MonitorMessageProcessor: Send + Sync + std::fmt::Debug + 'static {
    /// Process a message from a monitor thread, potentially generating internal
    /// messages from it.
    fn new(&self) -> Box<dyn MonitorMessageProcessorInstance>;
}

pub trait MonitorMessageProcessorInstance: Send + Sync + std::fmt::Debug + 'static {
    /// Process a message from a monitor thread, potentially generating internal
    /// messages from it.
    fn process_message(&self, input: &str) -> Vec<String>;

    fn finalize(&self) -> Vec<String>;
}

impl MonitorThread {
    /// Create a new monitor thread and release it
    fn create(
        monitor: MonitorDirConfig,
        mut state: MonitorState,
        css_config: CssMetadataConfig,
    ) -> Result<Self, Box<dyn Error>> {
        state.status.initialize(&css_config);
        for state in &mut state.children {
            state.1.status.initialize(&css_config);
        }
        let state = SharedMut::new(state);

        let monitor_state = state.clone();
        let drop_detect = SharedMut::new(());
        let mut drop_detect_clone = Some(drop_detect.clone());
        let _thread = thread::spawn(move || {
            monitor_thread(&monitor, move |id, m| {
                drop_detect_clone = if let Some(drop_detect) = drop_detect_clone.take() {
                    drop_detect.try_unwrap().err()
                } else {
                    None
                };

                if drop_detect_clone.is_none() {
                    return Err(ShuttingDown::default().into());
                }
                monitor_state
                    .write()
                    .process_message(id, m, &css_config, &mut |_| {})
            });
        });

        let thread = MonitorThread { state, drop_detect };

        Ok(thread)
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
            monitors: self.monitors.iter().map(|m| m.state.clone()).collect(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::worker::monitor_run;
    use std::path::Path;

    fn extract_status(status: &MonitorStatus) -> (StatusState, String, i64) {
        (
            status.status.unwrap(),
            status.description.clone(),
            status.code,
        )
    }

    fn extract_child_results(state: MonitorState) -> Vec<(StatusState, String, i64)> {
        state
            .children
            .iter()
            .map(|c| extract_status(&c.1.status))
            .collect()
    }

    fn run_test(test: &str) -> Result<MonitorState, Box<dyn Error>> {
        let config =
            parse_monitor_config(Path::new(&format!("src/testcases/{}/config.yaml", test)))?;
        let mut state: MonitorState = (&config).into();
        let metadata = CssMetadataConfig::default();
        monitor_run(&config, &mut |id, m| {
            state.process_message(id, m, &metadata, &mut |_| {})
        })
        .1?;
        Ok(state)
    }

    /// Test if metadata is set correctly when a script succeeds.
    #[test]
    fn metadata_success_test() -> Result<(), Box<dyn Error>> {
        use StatusState::*;
        let state = run_test("metadata_success")?;
        assert_eq!(
            extract_status(&state.status),
            (Yellow, "Custom (yellow)".into(), 0)
        );
        Ok(())
    }

    /// Test if metadata is not set when the script fails.
    #[test]
    fn metadata_fail_test() -> Result<(), Box<dyn Error>> {
        use StatusState::*;
        let state = run_test("metadata_fail")?;
        assert_eq!(extract_status(&state.status), (Red, "Failed".into(), 1));
        Ok(())
    }

    /// Tests if a complete group is correctly represented in the output.
    #[test]
    fn group_complete_test() -> Result<(), Box<dyn Error>> {
        use StatusState::*;
        let state = run_test("group_complete")?;
        assert_eq!(extract_status(&state.status), (Green, "Success".into(), 0));
        assert_eq!(
            extract_child_results(state),
            vec![
                (Yellow, "Success".into(), 0),
                (Green, "Success".into(), 0),
                (Yellow, "Success".into(), 0),
                (Red, "Success".into(), 0)
            ]
        );
        Ok(())
    }

    /// Test whether the group adopts the parent script's results when the script failed.
    #[test]
    fn group_fail_test() -> Result<(), Box<dyn Error>> {
        use StatusState::*;
        let state = run_test("group_fail")?;
        assert_eq!(extract_status(&state.status), (Red, "Failed".into(), 1));
        assert_eq!(
            extract_child_results(state),
            vec![
                (Red, "Failed".into(), 1),
                (Red, "Failed".into(), 1),
                (Red, "Failed".into(), 1),
                (Red, "Failed".into(), 1)
            ]
        );
        Ok(())
    }

    /// Tests whether the incomplete members of a group are correctly blanked out.
    #[test]
    fn group_incomplete_test() -> Result<(), Box<dyn Error>> {
        use StatusState::*;
        let state = run_test("group_incomplete")?;
        assert_eq!(extract_status(&state.status), (Green, "Success".into(), 0));
        assert_eq!(
            extract_child_results(state),
            vec![
                (Yellow, "Success".into(), 0),
                (Green, "Success".into(), 0),
                (Yellow, "Success".into(), 0),
            ]
        );
        Ok(())
    }
}
