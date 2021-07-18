#![windows_subsystem = "windows"]

use std::error::Error;
use std::fmt;
use std::io::{self, Read, Write};
use std::os::windows::process::CommandExt;
use std::process::ChildStderr;
use std::process::ChildStdout;
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use thiserror::Error;

fn output<R: Read + 'static + Send, F: 'static + Send + FnOnce() -> W, W: Write>(
    writer: F,
) -> mpsc::SyncSender<R> {
    let (tx, rx) = mpsc::sync_channel(0);
    thread::spawn(move || {
        let mut writer = writer();
        loop {
            let mut child_stdout = rx.recv().unwrap();
            io::copy(&mut child_stdout, &mut writer).unwrap();
        }
    });
    tx
}

#[derive(Debug, Error)]
enum AgentError {
    #[error("Failed to execute gpg-connect-agent")]
    Failed,
    #[error("gpg-connect-agent exited with code {0}")]
    Error(i32),
}

fn run(
    args: &[&str],
    stdout: &mpsc::SyncSender<ChildStdout>,
    stderr: &mpsc::SyncSender<ChildStderr>,
) -> Result<(), Box<dyn Error>> {
    let mut child = Command::new("gpg-connect-agent")
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()?;
    stdout.send(child.stdout.take().unwrap()).unwrap();
    stderr.send(child.stderr.take().unwrap()).unwrap();
    let status = child.wait()?;
    match status.code() {
        None => Err(Box::new(AgentError::Failed)),
        Some(0) => Ok(()),
        Some(code) => Err(Box::new(AgentError::Error(code))),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdout = output(io::stdout);
    let stderr = output(io::stderr);

    run(&["killagent", "/bye"], &stdout, &stderr)?;
    run(&["/bye"], &stdout, &stderr)?;

    Ok(())
}
