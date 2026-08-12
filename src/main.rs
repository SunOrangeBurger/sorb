mod shell;
mod easter_eggs;
mod chat;

fn main() {
    ctrlc::set_handler(move || {
        // No-op
    }).expect("Error setting Ctrl-C handler");

    shell::run();
}