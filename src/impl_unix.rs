use crate::Command;
use std::io::{Error, ErrorKind::NotFound};

/// Check if the current process is running with elevated privileges.
pub fn is_elevated() -> bool {
    unsafe { libc::geteuid() == 0 }
}

const PKEXEC: &str = "pkexec";
const SUDO: &str = "sudo";
const DOAS: &str = "doas";

/// Execute a command with elevated privileges using `pkexec`, `sudo`, or `doas`.
pub fn runas_impl(cmd: &Command) -> std::io::Result<std::process::ExitStatus> {
    if cmd.gui {
        #[cfg(all(unix, target_os = "linux"))]
        match which::which(PKEXEC) {
            Ok(_) => {
                let mut child = std::process::Command::new(PKEXEC);
                child.arg(&cmd.command).args(&cmd.args[..]).status()
            }
            Err(e) => Err(Error::new(NotFound, format!("Command {PKEXEC} not found: '{e}'"))),
        }
        #[cfg(all(unix, not(target_os = "linux")))]
        Err(Error::new(NotFound, format!("Command {PKEXEC} not found on non-Linux OS")))
    } else {
        let mut executor = None;
        if which::which(SUDO).is_ok() {
            executor = Some(SUDO);
        }
        // Detect if doas is installed and prefer using sudo
        if executor.is_none() && which::which(DOAS).is_ok() {
            executor = Some(DOAS);
        }
        match executor {
            Some(exec) => {
                let mut child = std::process::Command::new(exec);
                if exec == SUDO && cmd.force_prompt {
                    // Forces password re-prompting
                    child.arg("-k");
                }
                child.arg("--").arg(&cmd.command).args(&cmd.args[..]).status()
            }
            None => Err(Error::new(NotFound, format!("Commands {SUDO} or {DOAS} not found!"))),
        }
    }
}
