use ratatui::Frame;

/// Every minigame must implement this trait.
pub trait Game {
    /// Display name of the game.
    #[allow(dead_code)]
    fn name(&self) -> &str;

    /// Advance the game world by one tick.
    /// `input` is `Some(Input)` if the player pressed a key this frame.
    fn tick(&mut self, input: Option<Input>) -> GameState;

    /// Draw the current game state via ratatui.
    fn render(&self, frame: &mut Frame);

    /// Reset the game to its initial state.
    #[allow(dead_code)]
    fn reset(&mut self);
}

/// Represents the current state of a running game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameState {
    Running,
    #[allow(dead_code)]
    Paused,
    GameOver { score: u32 },
    QuitToShell,
}

/// Normalized input events consumed by games.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Input {
    Up,
    Down,
    Left,
    Right,
    Action,
    Quit,
}
