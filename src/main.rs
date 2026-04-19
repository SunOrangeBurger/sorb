mod shell;
mod easter_eggs;

fn main() {
    // Ignore Ctrl+C in the main thread to prevent the shell from dying
    // when a child process is interrupted.
    ctrlc::set_handler(move || {
        // No-op
    }).expect("Error setting Ctrl-C handler");

    shell::run();
}
