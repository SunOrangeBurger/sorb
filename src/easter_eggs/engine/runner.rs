use std::io::{self, stdout};
use std::time::{Duration, Instant};

use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use super::game::{Game, GameState};
use super::input::poll_input;
use super::scores;

/// Fixed tick duration (~60ms ≈ ~16.6 ticks/sec).
const TICK_DURATION: Duration = Duration::from_millis(60);

/// Runs a game in the alternate screen with raw mode enabled.
/// When the game ends or QuitToShell is reached, restores the terminal.
pub fn run_game(game: &mut dyn Game) {
    // --- Enter game mode ---
    enable_raw_mode().expect("Failed to enable raw mode");
    execute!(stdout(), EnterAlternateScreen).expect("Failed to enter alternate screen");

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
    terminal.clear().expect("Failed to clear terminal");

    // --- Game loop ---
    let result = game_loop(&mut terminal, game);

    // --- Restore terminal (always runs, even on panic via Drop guard below) ---
    restore_terminal();

    if let Err(e) = result {
        eprintln!("sorb: game crashed: {}", e);
    }
}

/// The core fixed-step game loop.
fn game_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    game: &mut dyn Game,
) -> io::Result<()> {
    loop {
        let tick_start = Instant::now();

        // 1. Poll input (use most of the tick budget for better responsiveness)
        let inputs = poll_input(Duration::from_millis(50));

        // 2. Advance game state - process all inputs
        let mut state = GameState::Running;
        if inputs.is_empty() {
            state = game.tick(None);
        } else {
            for input in inputs {
                state = game.tick(Some(input));
                // Stop processing if game wants to quit or is over
                if matches!(state, GameState::QuitToShell | GameState::GameOver { .. }) {
                    break;
                }
            }
        }

        // 3. Render
        terminal.draw(|frame| {
            game.render(frame);
        })?;

        // 4. Check state transitions
        match state {
            GameState::QuitToShell => break,
            GameState::GameOver { score } => {
                // Save score and check if it's a new high score
                let is_new_best = scores::save_score(game.name(), score);
                restore_terminal();
                println!("Game Over! Final score: {}", score);
                if is_new_best && score > 0 {
                    println!("🎉 NEW HIGH SCORE! 🎉");
                }
                return Ok(());
            }
            _ => {}
        }

        // 5. Sleep for the remainder of the tick
        let elapsed = tick_start.elapsed();
        if elapsed < TICK_DURATION {
            std::thread::sleep(TICK_DURATION - elapsed);
        }
    }

    Ok(())
}

/// Safely restores terminal to cooked mode and leaves the alternate screen.
fn restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(stdout(), LeaveAlternateScreen);
}
