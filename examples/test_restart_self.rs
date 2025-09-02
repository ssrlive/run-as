//! Example for re-executing the program itself
//!
//! This example demonstrates how to use the `restart_self` function with different
//! `wait_to_complete` settings and elevated restart function

use run_as::{restart_self, restart_self_elevated};
use std::env;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    println!("Program started, arguments: {:?}", args);

    // Check if this is already a restarted instance
    if args.len() > 1 && args[1] == "--restarted" {
        println!("This is the restarted instance!");
        println!("Waiting 5 seconds before exit...");
        thread::sleep(Duration::from_secs(5));
        return Ok(());
    }

    // Check if this is already an elevated instance
    if args.len() > 1 && args[1] == "--elevated" {
        println!("This is the elevated instance!");
        #[cfg(unix)]
        println!("Current user ID: {}", unsafe { libc::getuid() });
        println!("Waiting 5 seconds before exit...");
        thread::sleep(Duration::from_secs(5));
        return Ok(());
    }

    println!("Choose operation:");
    println!("1. Normal restart (non-blocking)");
    println!("2. Restart with elevated privileges (non-blocking)");
    println!("3. Restart and wait for completion (blocking)");
    println!("4. Restart with elevated privileges and wait (blocking)");
    println!("Please enter your choice (1, 2, 3, or 4):");

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    let choice = input.trim();

    match choice {
        "1" => {
            println!("Restarting program (non-blocking)...");
            let args = vec!["--restarted".to_string()];

            // Non-blocking restart
            match restart_self(Some(args), false) {
                Ok(None) => {
                    println!("Program restarted successfully (non-blocking).");
                    println!("Child process is now running independently.");
                    println!("Original program will exit in 2 seconds...");
                    thread::sleep(Duration::from_secs(2));
                }
                Ok(Some(_)) => {
                    println!("Unexpected: received exit status for non-blocking restart");
                }
                Err(e) => {
                    eprintln!("Non-blocking restart failed: {e}");
                }
            }
        }
        "2" => {
            println!("Restarting program with elevated privileges...");
            let args = vec!["--elevated".to_string()];

            // Non-blocking restart with elevated privileges
            match restart_self_elevated(Some(args), true, false) {
                Ok(None) => {
                    println!("Program restarted with elevated privileges (non-blocking)!");
                    println!("Original program will exit in 10 seconds...");
                    thread::sleep(Duration::from_secs(10));
                }
                Ok(Some(_)) => {
                    println!("Unexpected: received exit status for non-blocking elevated restart");
                }
                Err(e) => {
                    eprintln!("Elevated restart failed: {e}");
                }
            }
        }
        "3" => {
            println!("Restarting program and waiting for completion (blocking)...");
            let args = vec!["--restarted".to_string()];

            // Blocking restart - wait for the new process to complete
            match restart_self(Some(args), true) {
                Ok(Some(status)) => {
                    println!("Restarted process completed with status: {:?}", status);
                }
                Ok(None) => {
                    println!("This should not happen with wait_to_complete=true");
                }
                Err(e) => {
                    eprintln!("Blocking restart failed: {e}");
                }
            }
        }
        "4" => {
            println!("Restarting program with elevated privileges and waiting for completion...");
            let args = vec!["--elevated".to_string()];

            // Blocking restart with elevated privileges
            match restart_self_elevated(Some(args), true, true) {
                Ok(Some(status)) => {
                    println!("Elevated process completed with status: {:?}", status);
                }
                Ok(None) => {
                    println!("This should not happen with wait_to_complete=true");
                }
                Err(e) => {
                    eprintln!("Elevated blocking restart failed: {e}");
                }
            }
        }
        _ => {
            println!("Invalid choice");
        }
    }

    Ok(())
}
