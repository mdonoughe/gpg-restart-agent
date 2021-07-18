#![windows_subsystem = "windows"]

windows::include_bindings!();

use std::os::windows::process::CommandExt;
use std::process::Command;
use std::ptr;
use thiserror::Error;
use Windows::Win32::{
    Foundation::{PSTR, PWSTR},
    Globalization::{MultiByteToWideChar, WideCharToMultiByte, CP_ACP, CP_UTF8, MB_PRECOMPOSED},
    System::Threading::DETACHED_PROCESS,
    UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK},
};

#[derive(Debug, Error)]
enum CommandError<'a> {
    #[error("Failed to execute {0}: {1}")]
    Io(&'a str, std::io::Error),
    #[error("Failed to execute {0}")]
    Failed(&'a str),
    #[error("{program} exited with code {code}\nstdout:\n{stdout}\n\nstderr:\n{stderr}")]
    Error {
        program: &'a str,
        code: i32,
        stdout: String,
        stderr: String,
    },
}

#[derive(Debug, Error)]
enum RestartError<'a> {
    #[error("Failed to stop agent: {0}")]
    Stop(CommandError<'a>),
    #[error("Failed to start agent: {0}")]
    Start(CommandError<'a>),
}

fn run<'a>(program: &'a str, args: &[&str]) -> Result<(), CommandError<'a>> {
    let output = Command::new(program)
        .args(args)
        .creation_flags(DETACHED_PROCESS.0)
        .output()
        .map_err(|e| CommandError::Io(program, e))?;
    if output.status.success() {
        Ok(())
    } else {
        match output.status.code() {
            None => Err(CommandError::Failed(program)),
            Some(code) => Err(CommandError::Error {
                program,
                code,
                stdout: console_into_string(output.stdout),
                stderr: console_into_string(output.stderr),
            }),
        }
    }
}

fn console_into_string(mut bytes: Vec<u8>) -> String {
    unsafe {
        // Convert to new "Unicode" wide string.
        let len = MultiByteToWideChar(
            CP_ACP,
            MB_PRECOMPOSED,
            PSTR(bytes.as_ptr() as *mut _),
            bytes.len() as _,
            PWSTR(ptr::null_mut()),
            0,
        ) as usize;
        let mut wide = Vec::with_capacity(len);
        let len = MultiByteToWideChar(
            CP_ACP,
            MB_PRECOMPOSED,
            PSTR(bytes.as_ptr() as *mut _),
            bytes.len() as _,
            PWSTR(wide.as_mut_ptr()),
            len as _,
        ) as usize;
        wide.set_len(len);

        // Convert back into original buffer as UTF-8.
        bytes.clear();
        let len = bytes.capacity();
        let len = WideCharToMultiByte(
            CP_UTF8,
            0,
            PWSTR(wide.as_ptr() as *mut _),
            wide.len() as _,
            PSTR(bytes.as_mut_ptr()),
            len as _,
            PSTR(ptr::null_mut()),
            ptr::null_mut(),
        ) as usize;
        if len as usize > bytes.capacity() {
            bytes.reserve(len);
            WideCharToMultiByte(
                CP_UTF8,
                0,
                PWSTR(wide.as_ptr() as *mut _),
                wide.len() as _,
                PSTR(bytes.as_mut_ptr()),
                len as _,
                PSTR(ptr::null_mut()),
                ptr::null_mut(),
            );
        }
        bytes.set_len(len);

        String::from_utf8_unchecked(bytes)
    }
}

fn main_result() -> Result<(), RestartError<'static>> {
    run("gpgconf", &["--kill", "gpg-agent"]).map_err(RestartError::Stop)?;
    run("gpgconf", &["--launch", "gpg-agent"]).map_err(RestartError::Start)?;

    Ok(())
}

fn main() {
    if let Err(error) = main_result() {
        unsafe {
            MessageBoxW(
                None,
                format!("{}", error),
                "gpg-restart-agent",
                MB_OK | MB_ICONERROR,
            );
        }
    }
}
