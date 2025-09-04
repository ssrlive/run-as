//! Example for re-executing the program itself
//!
//! This example demonstrates how to use the `restart_self` function with different
//! `wait_to_complete` settings and elevated restart function

use run_as::{restart_self, restart_self_elevated};
use std::env;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();

    let args: Vec<String> = env::args().collect();

    log::debug!("Program started, arguments: {:?}", args);

    // Check if this is already a restarted instance
    if args.len() > 1 && args[1] == "--restarted" {
        log::info!("[Restarted] This is the restarted instance!");
        log::info!("[Restarted] Waiting 5 seconds before exit...");
        thread::sleep(Duration::from_secs(5));
        return Ok(());
    }

    // Check if this is already an elevated instance
    if args.len() > 1 && args[1] == "--elevated" {
        log::info!("[Elevated] This is the elevated instance!");
        #[cfg(unix)]
        log::info!("[Elevated] Current user ID: {}", unsafe { libc::getuid() });
        log::info!("[Elevated] Waiting 5 seconds before exit...");
        thread::sleep(Duration::from_secs(5));
        return Ok(());
    }

    log::info!("Choose operation:");
    log::info!("1. Normal restart (non-blocking)");
    log::info!("2. Restart with elevated privileges (non-blocking)");
    log::info!("3. Restart and wait for completion (blocking)");
    log::info!("4. Restart with elevated privileges and wait (blocking)");
    log::info!("Please enter your choice (1, 2, 3, or 4):");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let choice = input.trim();

    match choice {
        "1" => {
            log::info!("Restarting program (non-blocking)...");
            let args = vec!["--restarted".to_string()];

            // Non-blocking restart
            match restart_self(Some(args), false) {
                Ok(None) => {
                    log::info!("Program restarted successfully (non-blocking).");
                    log::info!("Child process is now running independently.");
                    log::info!("Original program will exit in 2 seconds...");
                    thread::sleep(Duration::from_secs(2));
                }
                Ok(Some(_)) => {
                    log::warn!("Unexpected: received exit status for non-blocking restart");
                }
                Err(e) => {
                    log::error!("Non-blocking restart failed: {e}");
                }
            }
        }
        "2" => {
            log::info!("Restarting program with elevated privileges...");
            let args = vec!["--elevated".to_string()];

            // Non-blocking restart with elevated privileges
            match restart_self_elevated(Some(args), true, false, None) {
                Ok(None) => {
                    log::info!("Program restarted with elevated privileges (non-blocking)!");
                    log::info!("Original program will exit in 10 seconds...");
                    thread::sleep(Duration::from_secs(10));
                }
                Ok(Some(_)) => {
                    log::warn!("Unexpected: received exit status for non-blocking elevated restart");
                }
                Err(e) => {
                    log::error!("Elevated restart failed: {e}");
                }
            }
        }
        "3" => {
            log::info!("Restarting program and waiting for completion (blocking)...");
            let args = vec!["--restarted".to_string()];

            // Blocking restart - wait for the new process to complete
            match restart_self(Some(args), true) {
                Ok(Some(status)) => {
                    log::info!("Restarted process completed with status: {:?}", status);
                }
                Ok(None) => {
                    log::warn!("This should not happen with wait_to_complete=true");
                }
                Err(e) => {
                    log::error!("Blocking restart failed: {e}");
                }
            }
        }
        "4" => {
            log::info!("Restarting program with elevated privileges and waiting for completion...");
            let args = vec!["--elevated".to_string()];

            // Blocking restart with elevated privileges
            match restart_self_elevated(Some(args), true, true, None) {
                Ok(Some(status)) => {
                    log::info!("Elevated process completed with status: {:?}", status);
                }
                Ok(None) => {
                    log::warn!("This should not happen with wait_to_complete=true");
                }
                Err(e) => {
                    log::error!("Elevated blocking restart failed: {e}");
                }
            }
        }
        _ => {
            log::warn!("Invalid choice");
        }
    }

    log::info!("Original program exiting.");

    Ok(())
}
