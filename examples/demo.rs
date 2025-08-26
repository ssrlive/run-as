#[cfg(windows)]
fn main() {
    println!("Is elevated: {}", run_as::is_elevated());
    println!("Running whoami /priv");
    println!(
        "Status: {}",
        run_as::Command::new("cmd")
            .arg("/k")
            .arg("whoami")
            .arg("/priv")
            .gui(true)
            .force_prompt(false)
            .wait_to_complete(true)
            .status()
            .expect("failed to execute")
    );
}

#[cfg(unix)]
fn main() {
    println!("Is elevated: {}", run_as::is_elevated());
    println!("Running id");
    println!(
        "Status: {}",
        run_as::Command::new("id")
            .gui(false)
            .wait_to_complete(true)
            .force_prompt(false)
            .status()
            .expect("failed to execute")
    );
}
