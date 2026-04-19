use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use super::game::Input;

/// Tracks which keys are currently held down and when they were last seen
pub struct InputState {
    held_keys: HashMap<Input, (u64, Instant)>, // Maps Input to (press_order, last_seen_time)
    press_counter: u64,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            held_keys: HashMap::new(),
            press_counter: 0,
        }
    }

    /// Poll for key events and update held key state
    /// Returns a Vec of inputs to process this frame
    pub fn poll(&mut self, timeout: Duration) -> Vec<Input> {
        let start = Instant::now();
        let mut any_key_seen = false;
        
        // Poll for new key events
        while start.elapsed() < timeout {
            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining.as_millis() == 0 {
                break;
            }
            
            if event::poll(remaining).unwrap_or(false) {
                if let Ok(Event::Key(KeyEvent {
                    code, modifiers, kind, ..
                })) = event::read()
                {
                    if let Some(input) = map_key(code, modifiers) {
                        match kind {
                            KeyEventKind::Press | KeyEventKind::Repeat => {
                                any_key_seen = true;
                                // Key pressed or repeated - update held keys
                                if !self.held_keys.contains_key(&input) {
                                    self.held_keys.insert(input, (self.press_counter, Instant::now()));
                                    self.press_counter += 1;
                                } else {
                                    // Update last seen time
                                    if let Some((order, _)) = self.held_keys.get(&input) {
                                        let order = *order;
                                        self.held_keys.insert(input, (order, Instant::now()));
                                    }
                                }
                            }
                            KeyEventKind::Release => {
                                // Key released - remove from held keys
                                self.held_keys.remove(&input);
                            }
                        }
                    }
                }
            } else {
                break;
            }
        }
        
        // If no keys were seen this frame, clear all held keys
        // This handles terminals that don't send release events
        if !any_key_seen {
            // Remove keys that haven't been seen for more than 100ms
            let now = Instant::now();
            self.held_keys.retain(|_, (_, last_seen)| {
                now.duration_since(*last_seen) < Duration::from_millis(100)
            });
        }
        
        // Generate inputs based on held keys
        self.generate_inputs()
    }

    fn generate_inputs(&self) -> Vec<Input> {
        if self.held_keys.is_empty() {
            return Vec::new();
        }

        // Sort held keys by press order
        let mut sorted_keys: Vec<(Input, u64)> = self.held_keys.iter()
            .map(|(k, (order, _))| (*k, *order))
            .collect();
        sorted_keys.sort_by_key(|(_, order)| *order);

        // Check for opposing inputs and filter them
        let filtered_keys = self.filter_opposing_inputs(sorted_keys);

        // Handle up to 3 inputs
        let keys_to_process: Vec<Input> = filtered_keys.into_iter()
            .take(3)
            .collect();

        match keys_to_process.len() {
            0 => Vec::new(),
            1 => {
                // Single key held - repeat it
                vec![keys_to_process[0]]
            }
            2 => {
                // Two keys held - return both (they'll be processed in order)
                keys_to_process
            }
            _ => {
                // Three keys held - return all three in press order
                keys_to_process
            }
        }
    }

    fn filter_opposing_inputs(&self, sorted_keys: Vec<(Input, u64)>) -> Vec<Input> {
        let mut result = Vec::new();
        let mut has_left = false;
        let mut has_right = false;
        let mut has_up = false;
        let mut has_down = false;

        for (input, _) in sorted_keys {
            match input {
                Input::Left => {
                    if !has_right {
                        result.push(input);
                        has_left = true;
                    }
                }
                Input::Right => {
                    if !has_left {
                        result.push(input);
                        has_right = true;
                    }
                }
                Input::Up => {
                    if !has_down {
                        result.push(input);
                        has_up = true;
                    }
                }
                Input::Down => {
                    if !has_up {
                        result.push(input);
                        has_down = true;
                    }
                }
                _ => {
                    // Non-directional inputs (Action, Quit) always pass through
                    result.push(input);
                }
            }
        }

        result
    }
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
