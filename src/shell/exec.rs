use std::process::Command;

/// Spawns an external command as a child process and waits for it to finish.
pub fn execute_external(cmd: &str, args: &[String]) {
    let mut child = match Command::new(cmd).args(args).spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!("sorb: command not found or failed to spawn: {} ({})", cmd, e);
            return;
        }
    };

    if let Err(e) = child.wait() {
        eprintln!("sorb: process executed with error: {}", e);
    }
}
