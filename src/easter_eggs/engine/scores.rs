use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Gets the path to the high scores file.
/// Ensures the `~/.sorb` directory exists.
fn get_scores_path() -> Option<PathBuf> {
    if let Some(mut path) = dirs::home_dir() {
        path.push(".sorb");
        if !path.exists() {
            let _ = fs::create_dir_all(&path);
        }
        path.push("scores.json");
        return Some(path);
    }
    None
}

/// Loads high scores from the JSON file.
fn load_scores() -> HashMap<String, u32> {
    if let Some(path) = get_scores_path() {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(scores) = serde_json::from_str(&content) {
                return scores;
            }
        }
    }
    HashMap::new()
}

/// Saves high scores to the JSON file.
fn write_scores(scores: &HashMap<String, u32>) {
    if let Some(path) = get_scores_path() {
        if let Ok(content) = serde_json::to_string_pretty(scores) {
            let _ = fs::write(path, content);
        }
    }
}

/// Compares a new score to the stored high score for a game.
/// If it's a new personal best, updates the file and returns `true`.
/// Otherwise, returns `false`.
pub fn save_score(game_name: &str, new_score: u32) -> bool {
    let mut scores = load_scores();
    let current_high = scores.get(game_name).copied().unwrap_or(0);

    if new_score > current_high {
        scores.insert(game_name.to_string(), new_score);
        write_scores(&scores);
        true
    } else {
        false
    }
}
