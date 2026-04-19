use ratatui::layout::{Alignment, Constraint, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::easter_eggs::engine::game::{Game, GameState, Input};

/// A dummy test game that verifies the engine hand-off works correctly.
/// It renders a simple message and quits when the user presses 'Q'.
pub struct TestGame {
    ticks: u32,
}

impl TestGame {
    pub fn new() -> Self {
        Self { ticks: 0 }
    }
}

impl Game for TestGame {
    fn name(&self) -> &str {
        "Engine Test"
    }

    fn tick(&mut self, input: Option<Input>) -> GameState {
        self.ticks += 1;

        if let Some(Input::Quit) = input {
            return GameState::QuitToShell;
        }

        GameState::Running
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::vertical([
            Constraint::Percentage(30),
            Constraint::Min(5),
            Constraint::Percentage(30),
        ])
        .split(area);

        let block = Block::default()
            .borders(Borders::ALL)
            .title(" 🎮 sorb – Engine Test ")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Cyan));

        let text = vec![
            Line::from(Span::styled(
                "✅ The game engine is working!",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("Ticks elapsed: {}", self.ticks),
                Style::default().fg(Color::Yellow),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press Q or Esc to return to the shell.",
                Style::default().fg(Color::Gray),
            )),
        ];

        let paragraph = Paragraph::new(text)
            .block(block)
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, chunks[1]);
    }

    fn reset(&mut self) {
        self.ticks = 0;
    }
}
