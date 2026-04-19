use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use std::time::Duration;

use super::game::Input;

/// Poll for crossterm key events within the given duration.
/// Returns a Vec of all recognized inputs that occurred.
pub fn poll_input(timeout: Duration) -> Vec<Input> {
    let mut inputs = Vec::new();
    let start = std::time::Instant::now();
    
    // Keep polling until timeout expires
    while start.elapsed() < timeout {
        let remaining = timeout.saturating_sub(start.elapsed());
        if remaining.as_millis() == 0 {
            break;
        }
        
        if event::poll(remaining).unwrap_or(false) {
            if let Ok(Event::Key(KeyEvent {
                code, modifiers, ..
            })) = event::read()
            {
                if let Some(input) = map_key(code, modifiers) {
                    inputs.push(input);
                }
            }
        } else {
            break;
        }
    }
    
    inputs
}

/// Map a raw crossterm key event to our normalized Input enum.
fn map_key(code: KeyCode, modifiers: KeyModifiers) -> Option<Input> {
    // Ctrl+C always quits
    if modifiers.contains(KeyModifiers::CONTROL) && code == KeyCode::Char('c') {
        return Some(Input::Quit);
    }

    match code {
        // Arrow keys
        KeyCode::Up => Some(Input::Up),
        KeyCode::Down => Some(Input::Down),
        KeyCode::Left => Some(Input::Left),
        KeyCode::Right => Some(Input::Right),

        // WASD mapping (games like Snake will use these)
        KeyCode::Char('w') | KeyCode::Char('W') => Some(Input::Up),
        KeyCode::Char('a') | KeyCode::Char('A') => Some(Input::Left),
        KeyCode::Char('s') | KeyCode::Char('S') => Some(Input::Down),
        KeyCode::Char('d') | KeyCode::Char('D') => Some(Input::Right),

        // Action key
        KeyCode::Char(' ') | KeyCode::Enter => Some(Input::Action),

        // Quit key
        KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc => Some(Input::Quit),

        _ => None,
    }
}
