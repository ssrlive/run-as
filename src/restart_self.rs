/// Re-execute the current program with configurable wait behavior
///
/// This function obtains the path of the currently executing program and restarts it.
/// You can control whether to wait for completion or run non-blocking.
///
/// # Arguments
///
/// * `args` - Optional additional command line arguments
/// * `wait_to_complete` - If true, waits for the process to complete; if false, returns immediately (non-blocking)
///
/// # Returns
///
/// Returns `Result<Option<std::process::ExitStatus>, std::io::Error>`:
/// - When `wait_to_complete` is true: `Some(ExitStatus)` with the process exit status
/// - When `wait_to_complete` is false: `None` (process runs in background)
///
/// # Examples
///
/// ```rust,no_run
/// use run_as::restart_self;
///
/// // Non-blocking restart
/// restart_self(None, false)?;
///
/// // Blocking restart (wait for completion)
/// if let Some(status) = restart_self(None, true)? {
///     println!("Process exited with: {:?}", status);
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn restart_self(args: Option<Vec<String>>, wait_to_complete: bool) -> std::io::Result<Option<std::process::ExitStatus>> {
    // Get the path of the current executable
    let current_exe = std::env::current_exe().map_err(|e| std::io::Error::other(format!("Failed to get current executable path: {e}")))?;

    let mut command = std::process::Command::new(current_exe);

    // Add original command line arguments (skip the first argument, which is the program name)
    command.args(std::env::args().skip(1));

    // Add additional arguments (if provided)
    if let Some(extra_args) = args {
        command.args(&extra_args);
    }

    if !wait_to_complete {
        // Configure the process to be detached from parent
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;

            unsafe {
                command.pre_exec(|| {
                    // Create a new session (this automatically creates a new process group too)
                    if libc::setsid() == -1 {
                        return Err(std::io::Error::last_os_error());
                    }

                    // Ignore hangup signal to survive terminal closure
                    if libc::signal(libc::SIGHUP, libc::SIG_IGN) == libc::SIG_ERR {
                        return Err(std::io::Error::last_os_error());
                    }

                    Ok(())
                });
            }

            // Redirect stdin, stdout, stderr to /dev/null to prevent blocking
            use std::process::Stdio;
            command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        }

        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // Detach from parent console on Windows
            command.creation_flags(0x00000008); // DETACHED_PROCESS

            use std::process::Stdio;
            command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null());
        }
    }

    if wait_to_complete {
        Ok(Some(command.spawn()?.wait()?)) // Ok(Some(command.status()?))
    } else {
        // Spawn new process non-blocking
        command.spawn().map(|_| None)
    }
}

/// Re-execute the current program with elevated privileges
///
/// This function re-executes the current program with elevated privileges.
/// You can control whether to wait for completion or run non-blocking.
///
/// # Arguments
///
/// * `args` - Optional additional command line arguments
/// * `gui` - Whether to use GUI mode for privilege elevation
/// * `wait_to_complete` - If true, waits for the process to complete; if false, returns immediately (non-blocking)
///
/// # Returns
///
/// Returns `Result<Option<std::process::ExitStatus>, std::io::Error>`:
/// - When `wait_to_complete` is true: `Some(ExitStatus)` with the process exit status
/// - When `wait_to_complete` is false: `None` (process runs in background)
///
/// # Examples
///
/// ```rust,no_run
/// use run_as::restart_self_elevated;
///
/// // Non-blocking restart with elevated privileges (command line mode)
/// restart_self_elevated(None, false, false, None)?;
///
/// // Blocking restart with elevated privileges (GUI mode, with additional arguments)
/// let args = vec!["--elevated".to_string()];
/// if let Some(status) = restart_self_elevated(Some(args), true, true, None)? {
///     println!("Elevated process exited with: {:?}", status);
/// }
/// # Ok::<(), std::io::Error>(())
/// ```
pub fn restart_self_elevated(
    args: Option<Vec<String>>,
    gui: bool,
    wait_to_complete: bool,
    _pkexec_timeout: Option<std::time::Duration>,
) -> std::io::Result<Option<std::process::ExitStatus>> {
    if crate::is_elevated() {
        // Already elevated, no need to restart
        return restart_self(args, wait_to_complete);
    }

    // Get the path of the current executable
    let current_exe = std::env::current_exe().map_err(|e| std::io::Error::other(format!("Failed to get current executable path: {e}")))?;

    let mut command = crate::Command::new(current_exe);

    // Configure command
    command.gui(gui).wait_to_complete(wait_to_complete);

    // Add original command line arguments (skip the first argument, which is the program name)
    command.args(std::env::args().skip(1));

    // Add additional arguments (if provided)
    if let Some(extra_args) = args {
        command.args(&extra_args);
    }

    #[cfg(target_os = "linux")]
    command.pkexec_timeout(_pkexec_timeout);

    // Execute command
    let status = command.status()?;

    if wait_to_complete { Ok(Some(status)) } else { Ok(None) }
}
