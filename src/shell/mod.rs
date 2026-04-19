pub mod builtins;
pub mod exec;

use std::env;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use crate::easter_eggs;

pub fn run() {
    let mut rl = DefaultEditor::new().expect("Failed to initialize rustyline");

    loop {
        // Render aesthetic colored prompt
        // \x1b[36m = Cyan, \x1b[32m = Green, \x1b[0m = Reset
        let prompt = if let Ok(cwd) = env::current_dir() {
            format!("\x1b[36msorb:\x1b[32m{}\x1b[0m $ ", cwd.display())
        } else {
            "\x1b[36msorb\x1b[0m $ ".to_string()
        };

        match rl.readline(&prompt) {
            Ok(line) => {
                let input = line.trim();
                if input.is_empty() {
                    continue;
                }

                // Add to history
                let _ = rl.add_history_entry(input);

                // Parse input robustly handling quotes
                match shell_words::split(input) {
                    Ok(words) => {
                        if words.is_empty() {
                            continue;
                        }
                        let cmd = &words[0];
                        let args = &words[1..];

                        // 1. Try built-ins first
                        if builtins::execute(cmd, args) {
                            continue;
                        }

                        // 2. Try easter egg commands
                        if easter_eggs::try_launch(cmd) {
                            continue;
                        }

                        // 3. Fallback to external process execution
                        exec::execute_external(cmd, args);
                    }
                    Err(e) => {
                        eprintln!("sorb: parse error: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                // Ctrl+C during prompt
                continue;
            }
            Err(ReadlineError::Eof) => {
                // Ctrl+D
                println!("exit");
                break;
            }
            Err(err) => {
                eprintln!("sorb: read error: {:?}", err);
                break;
            }
        }
    }
}
