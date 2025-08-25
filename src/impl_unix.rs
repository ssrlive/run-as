use std::io;
use std::process;
use std::process::ExitStatus;

use crate::Command;

pub fn runas_impl(cmd: &Command) -> io::Result<ExitStatus> {
    if cmd.gui {
        match which::which("pkexec") {
            Ok(_) => {
                let mut c = process::Command::new("pkexec");
                c.arg(&cmd.command).args(&cmd.args[..]).status()
            }
            Err(_) => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Command `pkexec` not found",
            )),
        }
    } else {
        match which::which("sudo") {
            Ok(_) => {
                let mut c = process::Command::new("sudo");
                if cmd.force_prompt {
                    c.arg("-k");
                }
                c.arg("--").arg(&cmd.command).args(&cmd.args[..]).status()
            }
            Err(_) => Err(io::Error::new(
                io::ErrorKind::NotFound,
                "Command `sudo` not found",
            )),
        }
    }
}
