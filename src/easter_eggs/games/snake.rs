use std::collections::VecDeque;

use rand::RngExt;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::easter_eggs::engine::game::{Game, GameState, Input};

/// Cardinal direction the snake is currently heading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// Returns the opposite direction (used to prevent 180° reversal).
    #[allow(dead_code)]
    fn opposite(self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

pub struct SnakeGame {
    /// Snake body segments. Front = head, back = tail.
    body: VecDeque<(u16, u16)>,
    /// Current movement direction.
    direction: Direction,
    /// Queued direction from latest input (applied on next tick to avoid double-move).
    next_direction: Direction,
    /// Current food position.
    food: (u16, u16),
    /// Play area dimensions (width, height) in cells.
    width: u16,
    height: u16,
    /// Current score.
    score: u32,
    /// Whether the game is over (waiting for user to see the score).
    game_over: bool,
}

impl SnakeGame {
    pub fn new() -> Self {
        let mut width = 40;
        let mut height = 20;
        if let Ok((term_w, term_h)) = crossterm::terminal::size() {
            width = (term_w.saturating_sub(2) / 2).max(10);
            height = term_h.saturating_sub(8).max(10);
        }

        let start_x = width / 2;
        let start_y = height / 2;

        let mut body = VecDeque::new();
        body.push_back((start_x, start_y));
        body.push_back((start_x.saturating_sub(1), start_y));
        body.push_back((start_x.saturating_sub(2), start_y));

        let mut game = Self {
            body,
            direction: Direction::Right,
            next_direction: Direction::Right,
            food: (0, 0),
            width,
            height,
            score: 0,
            game_over: false,
        };
        game.spawn_food();
        game
    }

    /// Place food at a random position not occupied by the snake.
    fn spawn_food(&mut self) {
        let mut free_spots = Vec::with_capacity((self.width * self.height) as usize);
        for y in 0..self.height {
            for x in 0..self.width {
                if !self.body.contains(&(x, y)) {
                    free_spots.push((x, y));
                }
            }
        }
        if !free_spots.is_empty() {
            let mut rng = rand::rng();
            self.food = free_spots[rng.random_range(0..free_spots.len())];
        }
    }
}

impl Game for SnakeGame {
    fn name(&self) -> &str {
        "Snake"
    }

    fn tick(&mut self, input: Option<Input>) -> GameState {
        // Handle input
        if let Some(inp) = input {
            match inp {
                Input::Quit => return GameState::QuitToShell,
                Input::Action if self.game_over => {
                    return GameState::GameOver { score: self.score };
                }
                Input::Up => {
                    if self.direction != Direction::Down {
                        self.next_direction = Direction::Up;
                    }
                }
                Input::Down => {
                    if self.direction != Direction::Up {
                        self.next_direction = Direction::Down;
                    }
                }
                Input::Left => {
                    if self.direction != Direction::Right {
                        self.next_direction = Direction::Left;
                    }
                }
                Input::Right => {
                    if self.direction != Direction::Left {
                        self.next_direction = Direction::Right;
                    }
                }
                _ => {}
            }
        }

        if self.game_over {
            return GameState::Running; // Keep rendering the game over screen
        }

        // Update dimensions if terminal resized
        if let Ok((term_w, term_h)) = crossterm::terminal::size() {
            let new_width = (term_w.saturating_sub(2) / 2).max(10);
            let new_height = term_h.saturating_sub(8).max(10);
            if new_width != self.width || new_height != self.height {
                self.width = new_width;
                self.height = new_height;
                // Clamp body and food to new boundaries
                for seg in self.body.iter_mut() {
                    seg.0 %= self.width;
                    seg.1 %= self.height;
                }
                self.food.0 %= self.width;
                self.food.1 %= self.height;
            }
        }

        // Apply queued direction
        self.direction = self.next_direction;

        // Calculate new head position with wrap-around
        let (hx, hy) = *self.body.front().unwrap();
        let new_head = match self.direction {
            Direction::Up => (hx, (hy + self.height - 1) % self.height),
            Direction::Down => (hx, (hy + 1) % self.height),
            Direction::Left => ((hx + self.width - 1) % self.width, hy),
            Direction::Right => ((hx + 1) % self.width, hy),
        };

        // Self collision (check before inserting new head)
        if self.body.contains(&new_head) {
            self.game_over = true;
            return GameState::Running;
        }

        // Move
        self.body.push_front(new_head);

        // Eat food or trim tail
        if new_head == self.food {
            self.score += 10;
            self.spawn_food();
        } else {
            self.body.pop_back();
        }

        GameState::Running
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();

        // Layout: top bar (3 lines), game area (fill), bottom bar (3 lines)
        let outer = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

        // --- Top bar: Score ---
        let score_text = if self.game_over {
            format!(
                "  Ororborus? or whatever they call it!  Score: {}  —  Press SPACE to continue, Q to quit  ",
                self.score
            )
        } else {
            format!(" SnakeekanS  |  Score: {}  |  Q to quit  ", self.score)
        };
        let score_bar = Paragraph::new(score_text)
            .style(
                Style::default()
                    .fg(Color::White)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
            .alignment(Alignment::Center)
            .block(Block::default());
        frame.render_widget(score_bar, outer[0]);

        // --- Game area ---
        let game_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Snake ")
            .title_alignment(Alignment::Center);

        let inner = game_block.inner(outer[1]);
        frame.render_widget(game_block, outer[1]);

        // Render the game content inside the bordered area
        self.render_game(frame, inner);

        // --- Bottom bar: Controls ---
        let controls = Paragraph::new("  ← ↑ ↓ → or WASD to move  |  Q / Esc to quit  ")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(controls, outer[2]);
    }

    fn reset(&mut self) {
        *self = SnakeGame::new();
    }
}

impl SnakeGame {
    /// Render snake body and food onto the game area.
    fn render_game(&self, frame: &mut Frame, area: Rect) {
        // We need to map game coordinates to the available terminal area.
        // Each game cell is 2 chars wide (for a more square appearance) and 1 char tall.
        let cell_width: u16 = 2;
        let cell_height: u16 = 1;

        // Offset to center the game within the area
        let total_game_w = self.width * cell_width;
        let total_game_h = self.height * cell_height;
        let offset_x = area.x + area.width.saturating_sub(total_game_w) / 2;
        let offset_y = area.y + area.height.saturating_sub(total_game_h) / 2;

        // Render food
        {
            let (fx, fy) = self.food;
            let px = offset_x + fx * cell_width;
            let py = offset_y + fy * cell_height;
            if px + cell_width <= area.x + area.width && py < area.y + area.height {
                let food_span = Span::styled("●", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
                let food_widget = Paragraph::new(Line::from(vec![food_span]));
                frame.render_widget(
                    food_widget,
                    Rect::new(px, py, cell_width, cell_height),
                );
            }
        }

        // Render snake body
        for (i, &(sx, sy)) in self.body.iter().enumerate() {
            let px = offset_x + sx * cell_width;
            let py = offset_y + sy * cell_height;
            if px + cell_width <= area.x + area.width && py < area.y + area.height {
                let color = if i == 0 {
                    Color::LightGreen // head
                } else {
                    Color::Green // body
                };
                let block_char = if i == 0 { "██" } else { "██" };
                let seg = Paragraph::new(Line::from(Span::styled(
                    block_char,
                    Style::default().fg(color),
                )));
                frame.render_widget(seg, Rect::new(px, py, cell_width, cell_height));
            }
        }
    }
}
