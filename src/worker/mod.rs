use std::error::Error;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use subprocess::{Exec, ExitStatus, Popen, Redirection};

use self::linebuf::LineBuf;
use crate::config::*;

mod linebuf;

#[derive(Debug, Default, Display, Error)]
pub struct ShuttingDown {}

#[derive(Debug)]
pub enum LogStream {
    StdOut,
    StdErr,
}

#[derive(Debug)]
pub enum WorkerMessage {
    Starting,
    LogMessage(LogStream, String),
    Metadata(String),
    Termination(i64),
    AbnormalTermination(String),
}

pub fn monitor_thread<T: FnMut(&str, WorkerMessage) -> Result<(), Box<dyn Error>>>(
    monitor: MonitorDirConfig,
    mut sender: T,
) {
    loop {
        let args: Option<&[OsString]> = None;
        let test = match monitor.root {
            MonitorDirRootConfig::Test(ref test) => test,
            MonitorDirRootConfig::Group(ref group) => &group.test,
        };
        let r = monitor_thread_impl(
            &monitor.id,
            &test.command,
            &monitor.base_path,
            args,
            test.timeout,
            &mut sender,
        );
        if let Err(err) = r {
            // Break the loop on a task failure (but don't log ShuttingDown errors)
            if err.downcast_ref::<ShuttingDown>().is_none() {
                error!("[{}] Task failure: {}", monitor.id, err);
            }
            if sender(
                &monitor.id,
                WorkerMessage::AbnormalTermination(err.to_string()),
            )
            .is_err()
            {
                return;
            }
        }
        trace!("[{}] Sleeping {}ms", monitor.id, test.interval.as_millis());
        thread::sleep(test.interval);
    }
}

fn append<T: FnMut(LogStream, String)>(
    id: &str,
    f: &mut T,
    out: &mut Option<Vec<u8>>,
    stdout: &mut LineBuf,
    err: &mut Option<Vec<u8>>,
    stderr: &mut LineBuf,
) -> bool {
    if let (Some(out), Some(err)) = (out, err) {
        // Termination condition: "until read() returns all-empty data, which marks EOF."
        let done = out.is_empty() && err.is_empty();
        if !done {
            // This is pretty noisy, so only trace if we have data
            trace!("[{}] read out={} err={}", id, out.len(), err.len());
        }
        stdout.accept(out, &mut |s| f(LogStream::StdOut, s));
        stderr.accept(err, &mut |s| f(LogStream::StdErr, s));
        done
    } else {
        error!("[{}] null reader?", id);
        debug_assert!(false, "Unexpectedly null reads");
        false
    }
}

enum DeathResult {
    ExitStatus(ExitStatus),
    Wedged(Popen),
}

fn aggressively_wait_for_death(id: &str, mut popen: Popen, duration: Duration) -> DeathResult {
    let r = popen.wait_timeout(duration);
    if let Ok(Some(status)) = r {
        // Easy, status was available right await
        debug!("[{}] Normal exit: {:?}", id, status);
        return DeathResult::ExitStatus(status);
    }

    // If we didn't get a result OR there was an error, let's try to terminate the process, ignoring any errors
    info!("[{}] Terminating process...", id);
    let _ = popen.terminate();

    // Now give it 5 seconds to exit for good
    let r = popen.wait_timeout(Duration::from_millis(5000));
    if let Ok(Some(_)) = r {
        // Always return the signal
        return DeathResult::ExitStatus(ExitStatus::Signaled(1));
    }

    // Kill with prejudice
    info!("[{}] Killing process...", id);
    let _ = popen.kill();

    // Give it another 5 seconds
    let r = popen.wait_timeout(Duration::from_millis(5000));
    if let Ok(Some(_)) = r {
        // Always return the signal
        return DeathResult::ExitStatus(ExitStatus::Signaled(9));
    }

    // This process is probably wedged and will become a zombie
    error!("[{}] Process wedged, bad things may happen", id);
    DeathResult::Wedged(popen)
}

fn process_log_message<T: FnMut(&str, WorkerMessage) -> Result<(), Box<dyn Error>>>(
    id: &str,
    failed: &AtomicBool,
    stream: LogStream,
    s: String,
    sender: &mut T,
) {
    const META_PREFIX: &str = "@@STYLUS@@";

    let msg = if s.starts_with(META_PREFIX) {
        let s = s.split_at(META_PREFIX.len()).1;
        WorkerMessage::Metadata(s.trim().to_owned())
    } else {
        WorkerMessage::LogMessage(stream, s)
    };

    if sender(id, msg).is_err() {
        failed.store(true, Ordering::SeqCst)
    }
}

fn monitor_thread_impl<T: FnMut(&str, WorkerMessage) -> Result<(), Box<dyn Error>>>(
    id: &str,
    cmd: &Path,
    base_path: &Path,
    args: Option<&[impl AsRef<OsStr>]>,
    timeout: Duration,
    sender: &mut T,
) -> Result<(), Box<dyn Error>> {
    // This will fail if we're supposed to shut down
    sender(id, WorkerMessage::Starting)?;

    debug!("[{}] Starting {:?}", id, cmd);

    let mut exec = Exec::cmd(cmd)
        .cwd(base_path)
        .env("STYLUS_MONITOR_ID", id)
        .stdout(Redirection::Pipe)
        .stderr(Redirection::Pipe);
    if let Some(args) = args {
        exec = exec.args(args);
    }
    let mut popen = exec.popen()?;

    let failed = AtomicBool::new(false);
    let mut f = |stream, s| {
        process_log_message(id, &failed, stream, s, sender);
    };
    let mut stdout = LineBuf::new(80);
    let mut stderr = LineBuf::new(80);

    let start = Instant::now();
    let mut comms = popen
        .communicate_start(None)
        .limit_time(Duration::from_millis(250));

    while start.elapsed() < timeout {
        let mut r = comms.read();
        if let Err(ref mut err) = r {
            if err.error.kind() == std::io::ErrorKind::TimedOut {
                if append(
                    id,
                    &mut f,
                    &mut err.capture.0,
                    &mut stdout,
                    &mut err.capture.1,
                    &mut stderr,
                ) {
                    // We *might* have a completed process: need to check whether the return value is available or not
                    if popen.poll().is_some() {
                        debug!("[{}] Early completion", id);
                        break;
                    }
                }
                continue;
            }
        }
        let mut r = r?;
        if append(id, &mut f, &mut r.0, &mut stdout, &mut r.1, &mut stderr) {
            break;
        }
    }

    stdout.close(&mut |s| f(LogStream::StdOut, s));
    stderr.close(&mut |s| f(LogStream::StdErr, s));

    debug!("[{}] Finished read, waiting for status...", id);

    // Give the process reaper at least 250ms to get the exit code (or longer if the test timeout is still not elapsed)
    let timeout = Duration::max(
        Duration::from_millis(250),
        timeout
            .checked_sub(start.elapsed())
            .unwrap_or(Duration::from_secs(0)),
    );
    match aggressively_wait_for_death(id, popen, timeout) {
        DeathResult::ExitStatus(ExitStatus::Exited(code)) => {
            sender(id, WorkerMessage::Termination(code as i64))?;
        }
        DeathResult::ExitStatus(ExitStatus::Signaled(code)) => {
            sender(
                id,
                WorkerMessage::AbnormalTermination(format!("Process exited with signal {}", code)),
            )?;
        }
        DeathResult::ExitStatus(ExitStatus::Other(code)) => {
            sender(
                id,
                WorkerMessage::AbnormalTermination(format!(
                    "Process exited for unknown reason {:x}",
                    code
                )),
            )?;
        }
        DeathResult::ExitStatus(ExitStatus::Undetermined) => {
            sender(
                id,
                WorkerMessage::AbnormalTermination("Process exited for unknown reason".into()),
            )?;
        }
        DeathResult::Wedged(mut popen) => {
            sender(
                id,
                WorkerMessage::AbnormalTermination("Process timed out".into()),
            )?;
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
        monitor_thread_impl(
            &"test".to_owned(),
            Path::new("/bin/sleep"),
            Path::new("/tmp"),
            Some(&["10"]),
            Duration::from_millis(250),
            &mut |_, m| {
                tx.send(m)?;
                Ok(())
            },
        )
        .expect("Failed to monitor");
        drop(tx);
        loop {
            if let Ok(msg) = rx.recv() {
                if let WorkerMessage::AbnormalTermination(_) = msg {
                    return;
                }
            } else {
                panic!("Never got the abnormal termination error")
            }
        }
    }
}
