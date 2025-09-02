use crate::Command;
use std::io::{Error, ErrorKind::NotFound};
use std::os::unix::process::ExitStatusExt;

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
                child.arg(&cmd.command).args(&cmd.args[..]);
                if cmd.wait_to_complete {
                    child.status()
                } else {
                    // FIXME: The non-blocking calling to `pkexec` will be killed by parent process while it exiting.
                    // This issue must be fixed in the future. But for now, I'm not find out how to solve it yet.

                    use std::os::unix::process::CommandExt;

                    // Create new process group and session for complete detachment
                    // child.process_group(0);

                    unsafe {
                        child.pre_exec(|| {
                            // Create a new process group (detach from parent's process group)
                            // if libc::setpgid(0, 0) == -1 {
                            //     return Err(std::io::Error::last_os_error());
                            // }

                            // Create a new session (this automatically creates a new process group too)
                            if libc::setsid() == -1 {
                                return Err(std::io::Error::last_os_error());
                            }

                            // Ignore hangup signal to survive terminal closure
                            libc::signal(libc::SIGHUP, libc::SIG_IGN);

                            Ok(())
                        });
                    }

                    // Redirect stdin, stdout, stderr to /dev/null to prevent blocking
                    use std::process::Stdio;
                    child.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());

                    child.spawn().map(|_| std::process::ExitStatus::from_raw(0))
                }
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
                child.arg("--").arg(&cmd.command).args(&cmd.args[..]);
                if cmd.wait_to_complete {
                    child.status()
                } else {
                    use std::os::unix::process::CommandExt;

                    unsafe {
                        child.pre_exec(|| {
                            // Create a new session (this automatically creates a new process group too)
                            if libc::setsid() == -1 {
                                return Err(std::io::Error::last_os_error());
                            }

                            // Ignore hangup signal to survive terminal closure
                            libc::signal(libc::SIGHUP, libc::SIG_IGN);

                            Ok(())
                        });
                    }

                    // Redirect stdin, stdout, stderr to /dev/null to prevent blocking
                    use std::process::Stdio;
                    child.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());

                    child.spawn().map(|_| std::process::ExitStatus::from_raw(0))
                }
            }
            None => Err(Error::new(NotFound, format!("Commands {SUDO} or {DOAS} not found!"))),
        }
    }
}
