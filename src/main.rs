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

const CREATE_NO_WINDOW: u32 = 0x08000000;

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

#[derive(Debug)]
enum AgentError {
    Failed,
    Error(i32),
}

impl Error for AgentError {}

impl fmt::Display for AgentError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            AgentError::Failed => write!(f, "Failed to execute gpg-connect-agent"),
            AgentError::Error(code) => write!(f, "gpg-connect-agent exited with code {}", code),
        }
    }
}

fn run(
    args: &[&str],
    stdout: &mpsc::SyncSender<ChildStdout>,
    stderr: &mpsc::SyncSender<ChildStderr>,
) -> Result<(), Box<Error>> {
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

fn main() -> Result<(), Box<Error>> {
    let stdout = output(io::stdout);
    let stderr = output(io::stderr);

    run(&["killagent", "/bye"], &stdout, &stderr)?;
    run(&["/bye"], &stdout, &stderr)?;

    Ok(())
}
