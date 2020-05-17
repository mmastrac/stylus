use std::error::Error;
use std::path::Path;
use std::thread;
use std::time::{Duration, Instant};
use std::ffi::{OsStr, OsString};

use subprocess::{Exec, ExitStatus, Popen, Redirection};

use crate::config::*;

#[derive(Debug)]
pub enum WorkerMessage {
    Starting,
    LogMessage(String),
    Termination(i64),
    AbnormalTermination(String),
}

pub fn monitor_thread<T: FnMut(WorkerMessage) -> Result<(), Box<dyn Error>>>(monitor: MonitorDirConfig, mut sender: T) {
    loop {
        let args: Option<&[OsString]> = None;
        let _ = monitor_thread_impl(&monitor.test.script, args, monitor.test.timeout, &mut sender);
        thread::sleep(monitor.test.interval);
    }
}

fn append(
    out: &mut Option<Vec<u8>>,
    stdout: &mut Vec<u8>,
    err: &mut Option<Vec<u8>>,
    stderr: &mut Vec<u8>,
) -> bool {
    if let (Some(out), Some(err)) = (out, err) {
        // Termination condition: "until read() returns all-empty data, which marks EOF."
        let done = out.len() == 0 && err.len() == 0;
        stdout.append(out);
        stderr.append(err);
        done
    } else {
        debug_assert!(false, "Unexpectedly null reads");
        false
    }
}

enum DeathResult {
    ExitStatus(ExitStatus),
    Wedged(Popen),
}

fn aggressively_wait_for_death(mut popen: Popen, duration: Duration) -> DeathResult {
    let r = popen.wait_timeout(duration);
    if let Ok(Some(status)) = r {
        // Easy, status was available right await
        return DeathResult::ExitStatus(status);
    }

    // If we didn't get a result OR there was an error, let's try to terminate the process, ignoring any errors
    let _ = popen.terminate();

    // Now give it 5 seconds to exit for good
    let r = popen.wait_timeout(Duration::from_millis(5000));
    if let Ok(Some(_)) = r {
        // Always return the signal
        return DeathResult::ExitStatus(ExitStatus::Signaled(1));
    }

    // Kill with prejudice
    let _ = popen.kill();

    // Give it another 5 seconds
    let r = popen.wait_timeout(Duration::from_millis(5000));
    if let Ok(Some(_)) = r {
        // Always return the signal
        return DeathResult::ExitStatus(ExitStatus::Signaled(9));
    }

    // This process is probably wedged and will become a zombie
    DeathResult::Wedged(popen)
}

fn monitor_thread_impl<T: FnMut(WorkerMessage) -> Result<(), Box<dyn Error>>>(
    cmd: &Path,
    args: Option<&[impl AsRef<OsStr>]>,
    timeout: Duration,
    sender: &mut T,
) -> Result<(), Box<dyn Error>> {
    // This will fail if we're supposed to shut down
    sender(WorkerMessage::Starting)?;

    debug!("Starting {:?}", cmd);

    let mut exec = Exec::cmd(cmd)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Pipe);
    if let Some(args) = args {
        exec = exec.args(args);
    }
    let mut popen = exec.popen()?;

    // TODO: We don't actually send log messages
    sender(WorkerMessage::LogMessage("TODO".into()))?;

    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let start = Instant::now();
    let mut comms = popen
        .communicate_start(None)
        .limit_time(Duration::from_millis(250));
    while start.elapsed() < timeout {
        let mut r = comms.read();
        if let Err(ref mut err) = r {
            if err.error.kind() == std::io::ErrorKind::TimedOut {
                append(
                    &mut err.capture.0,
                    &mut stdout,
                    &mut err.capture.1,
                    &mut stderr,
                );
                continue;
            }
        }
        let mut r = r?;
        if append(&mut r.0, &mut stdout, &mut r.1, &mut stderr) {
            break;
        }
    }

    debug!("Finished read, waiting for status...");

    // Give the process reaper at least 250ms to get the exit code (or longer if the test timeout is still not elapsed)
    let timeout = Duration::max(Duration::from_millis(250), timeout.checked_sub(start.elapsed()).unwrap_or(Duration::from_secs(0)));
    match aggressively_wait_for_death(popen, timeout) {
        DeathResult::ExitStatus(ExitStatus::Exited(code)) => {
            sender(WorkerMessage::Termination(code as i64))?;
        }
        DeathResult::ExitStatus(ExitStatus::Signaled(code)) => {
            sender(WorkerMessage::AbnormalTermination(
                format!("Process exited with signal {}", code).into(),
            ))?;
        }
        DeathResult::ExitStatus(ExitStatus::Other(code)) => {
            sender(WorkerMessage::AbnormalTermination(
                format!("Process exited for unknown reason {:x}", code).into(),
            ))?;
        }
        DeathResult::ExitStatus(ExitStatus::Undetermined) => {
            sender(WorkerMessage::AbnormalTermination(
                "Process exited for unknown reason".into(),
            ))?;
        }
        DeathResult::Wedged(mut popen) => {
            sender(WorkerMessage::AbnormalTermination(
                "Process timed out".into(),
            ))?;
            // We can wait here after we notify the monitor system
            let _ = popen.wait();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc::*;

    #[test]
    fn test_timeout() {
        let (tx, rx) = channel();
        monitor_thread_impl(Path::new("/bin/sleep"), Some(&["10"]), Duration::from_millis(5000), &mut |m| { tx.send(m)?; Ok(()) })
            .expect("Failed to monitor");
        drop(tx);
        loop {
            if let Ok(msg) = rx.recv() {
                println!("{:?}", msg);
            } else {
                break;
            }
        }
    }
}
