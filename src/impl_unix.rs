use crate::Command;
use std::io::{Error, ErrorKind::NotFound};

/// Check if the current process is running with elevated privileges.
pub fn is_elevated() -> bool {
    unsafe { libc::geteuid() == 0 }
}

pub fn runas_impl(cmd: &Command) -> std::io::Result<std::process::ExitStatus> {
    if cmd.gui {
        match which::which("pkexec") {
            Ok(_) => {
                let mut c = std::process::Command::new("pkexec");
                c.arg(&cmd.command).args(&cmd.args[..]).status()
            }
            Err(e) => Err(Error::new(NotFound, format!("`pkexec` not found: '{e}'"))),
        }
    } else {
        let mut executor = None;
        if which::which("sudo").is_ok() {
            executor = Some("sudo");
        }
        // Detect if doas is installed and prefer using sudo
        if executor.is_none() && which::which("doas").is_ok() {
            executor = Some("doas");
        }
        match executor {
            Some(exec) => {
                let mut c = std::process::Command::new(exec);
                if exec == "sudo" && cmd.force_prompt {
                    // Forces password re-prompting
                    c.arg("-k");
                }
                c.arg("--").arg(&cmd.command).args(&cmd.args[..]).status()
            }
            None => Err(Error::new(NotFound, "Commands sudo or doas not found!")),
        }
    }
}
