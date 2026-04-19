use std::env;
use std::path::Path;

/// Executes a built-in command if it matches.
/// Returns `true` if the command was handled (whether successful or not),
/// and `false` if it is not a recognized built-in.
pub fn execute(cmd: &str, args: &[String]) -> bool {
    match cmd {
        "cd" => {
            let target = args.get(0).map(|s| s.as_str()).unwrap_or("~");
            let path = if target == "~" {
                dirs::home_dir().unwrap_or_else(|| Path::new("/").to_path_buf())
            } else {
                Path::new(target).to_path_buf()
            };

            if let Err(e) = env::set_current_dir(&path) {
                eprintln!("cd: {}: {}", path.display(), e);
            }
            true
        }
        "exit" => {
            std::process::exit(0);
        }
        _ => false,
    }
}
