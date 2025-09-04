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
                // xhost +SI:localuser:root
                std::process::Command::new("xhost").arg("+SI:localuser:root").status()?;

                let mut child = std::process::Command::new(PKEXEC);

                // pkexec env DISPLAY=$DISPLAY XAUTHORITY=$XAUTHORITY /home/my/gui-app/main-exe
                child.arg("env");
                child.arg(format!("DISPLAY={}", std::env::var("DISPLAY").unwrap_or_default()));
                child.arg(format!("XAUTHORITY={}", std::env::var("XAUTHORITY").unwrap_or_default()));

                child.arg(&cmd.command).args(&cmd.args[..]);

                if cmd.wait_to_complete {
                    child.status()
                } else {
                    /*
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
                    // */

                    let timeout = cmd.pkexec_timeout.unwrap_or(crate::PKEXEC_TIMEOUT);
                    let command_path = std::path::PathBuf::from(&cmd.command);
                    std::thread::spawn(move || {
                        if monitor_root_process_startup(&command_path, timeout) {
                            // Successfully monitored root process startup.
                            // To avoid pkexec killed it, let thread sleep for a short duration
                            std::thread::sleep(std::time::Duration::from_millis(100));
                        }
                        // FIXME: Here we exit the caller process forcefully, but maybe it's not the expected behavior
                        std::process::exit(0);
                    });

                    // Can't use `child.spawn()` because we need to monitor the root process startup
                    child.status()
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

/// Workaround for root process detection.
///
/// pkexec will kill its child process if the parent exits before the child is fully started as root.
/// To avoid this, we spawn a thread that repeatedly scans /proc for a process whose executable name matches
/// `command_name` and whose UID is 0 (root). Once such a process is detected, we assume pkexec has finished
/// privilege escalation and the GUI process is running as root.
/// This moment we can allow the parent process to exit without pkexec killing the child.
///
/// This approach is more reliable than a fixed sleep, as it adapts to the actual startup time of the child process.
/// If the process is not detected within the timeout, the thread exits anyway.
///
/// Monitors /proc for a process with the given executable name and UID 0 (root).
/// Returns when such a process is found or when the timeout is reached.
/// Caller is responsible for any follow-up actions, such as exiting the process.
///
/// Limitations:
/// - If the child process fails to start, the function will wait for the timeout.
/// - If multiple processes with the same name run as root, this may cause false positives.
/// - Only works for Linux systems with /proc available.
#[cfg(target_os = "linux")]
fn monitor_root_process_startup(command_path: &std::path::Path, timeout: std::time::Duration) -> bool {
    let mut success = false;
    let start = std::time::Instant::now();
    // Wait at most timeout duration
    'exit_point: while start.elapsed() < timeout {
        // let mut found = false;
        if let Ok(read_dir) = std::fs::read_dir("/proc") {
            for entry in read_dir.flatten() {
                let file_name = entry.file_name();
                let pid_str = file_name.to_string_lossy();
                let pid = match pid_str.parse::<i32>() {
                    Ok(pid) => pid,
                    Err(_) => continue,
                };

                // 1. Try to get executable path from /proc/[pid]/cmdline (first argument)
                let cmdline_path = format!("/proc/{pid}/cmdline");
                let cmdline = std::fs::read(&cmdline_path).unwrap_or_default();
                // cmdline is null-separated
                let first_arg = cmdline.split(|&b| b == 0).next().unwrap_or(&[]);
                let first_arg_str = std::str::from_utf8(first_arg).unwrap_or("");
                let first_arg_path = std::path::Path::new(first_arg_str);
                let compare = if first_arg_path.is_absolute() && command_path.is_absolute() {
                    first_arg_path == command_path
                } else {
                    match (first_arg_path.file_name(), command_path.file_name()) {
                        (Some(a), Some(b)) => a == b,
                        _ => false,
                    }
                };
                if !compare {
                    continue;
                }
                log::trace!("Executable path: {first_arg_path:?}");

                // 2. Read process status and check UID
                let status_path = format!("/proc/{pid}/status");
                let status = match std::fs::read_to_string(&status_path) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                if status.lines().any(|line| {
                    if line.starts_with("Uid:") {
                        log::trace!("Pid: {pid}, line: {line}");
                        if line.split_whitespace().nth(1) == Some("0") {
                            return true;
                        }
                    }
                    false
                }) {
                    // Found root process, break loop and return
                    log::info!("Found root process: {pid}");
                    success = true;
                    break 'exit_point;
                }
            }
        }
        // Not found, sleep 1 second and retry
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    // Function returns when root process is found or timeout is reached
    success
}
