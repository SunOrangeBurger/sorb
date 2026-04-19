use std::collections::VecDeque;

use rand::RngExt;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::easter_eggs::engine::game::{Game, GameState, Input};

const GRAVITY: f32 = 0.25;
const FLAP_FORCE: f32 = 1.0;

pub struct FlappyGame {
    bird_y: f32,
    jump_start_y: f32,
    ticks_since_jump: f32,
    initial_velocity: f32,
    /// x, gap_y, gap_size
    pipes: VecDeque<(f32, f32, f32)>, 
    score: u32,
    game_over: bool,
    width: u16,
    height: u16,
    ticks_since_pipe: u32,
    pipe_speed: f32,
    gap_size: f32,
    game_started: bool,
}

impl FlappyGame {
    pub fn new() -> Self {
        let mut width = 40;
        let mut height = 20;
        if let Ok((term_w, term_h)) = crossterm::terminal::size() {
            width = (term_w.saturating_sub(2) / 2).max(10);
            height = term_h.saturating_sub(8).max(10);
        }

        Self {
            bird_y: height as f32 / 2.0,
            jump_start_y: height as f32 / 2.0,
            ticks_since_jump: 0.0,
            initial_velocity: 0.0,
            pipes: VecDeque::new(),
            score: 0,
            game_over: false,
            width,
            height,
            ticks_since_pipe: 0,
            pipe_speed: 0.5,
            gap_size: 6.0,
            game_started: false,
        }
    }
}

impl Game for FlappyGame {
    fn name(&self) -> &str {
        "Flappy Bird"
    }

    fn tick(&mut self, input: Option<Input>) -> GameState {
        if let Ok((term_w, term_h)) = crossterm::terminal::size() {
            self.width = (term_w.saturating_sub(2) / 2).max(10);
            self.height = term_h.saturating_sub(8).max(10);
        }

        if let Some(inp) = input {
            match inp {
                Input::Quit => return GameState::QuitToShell,
                Input::Action => {
                    if self.game_over {
                        return GameState::GameOver { score: self.score };
                    } else {
                        if !self.game_started {
                            self.game_started = true;
                        }
                        self.jump_start_y = self.bird_y;
                        self.ticks_since_jump = 0.0;
                        self.initial_velocity = -FLAP_FORCE * 1.5; // Scale jump velocity for quadratic arc
                    }
                }
                _ => {}
            }
        }

        if self.game_over {
            return GameState::Running;
        }

        // Auto-jump at start to prevent instant death
        if !self.game_started {
            self.jump_start_y = self.bird_y;
            self.ticks_since_jump = 0.0;
            self.initial_velocity = -FLAP_FORCE * 1.5;
            self.game_started = true;
        }

        // Apply quadratic physics arc: y = y0 + v0*t + 0.5*a*t^2
        self.ticks_since_jump += 1.0;
        self.bird_y = self.jump_start_y 
                      + (self.initial_velocity * self.ticks_since_jump)
                      + (0.5 * GRAVITY * self.ticks_since_jump * self.ticks_since_jump);

        // Strict floor/ceiling collision based on actual height
        if self.bird_y >= self.height as f32 - 1.0 {
            self.bird_y = self.height as f32 - 1.0;
            self.game_over = true;
            return GameState::Running;
        }
        if self.bird_y < 0.0 {
            self.bird_y = 0.0;
            self.game_over = true;
            return GameState::Running;
        }

        // Move pipes
        let mut score_increased = false;
        for p in self.pipes.iter_mut() {
            let old_x = p.0;
            p.0 -= self.pipe_speed;
            // Passed pipe?
            if old_x >= 10.0 && p.0 < 10.0 {
                score_increased = true;
            }
        }
        
        if score_increased {
            self.score += 1;
            
            // Increase difficulty based on score
            self.pipe_speed = match self.score {
                0..=4 => 0.5,
                5..=9 => 0.6,
                10..=14 => 0.7,
                15..=19 => 0.8,
                20..=29 => 0.9,
                _ => 1.0,
            };
            
            self.gap_size = match self.score {
                0..=9 => 6.0,
                10..=19 => 5.5,
                20..=29 => 5.0,
                _ => 4.5,
            };
        }

        // Remove off-screen pipes
        if let Some(front) = self.pipes.front() {
            if front.0 < -2.0 {
                self.pipes.pop_front();
            }
        }

        // Spawn pipes
        self.ticks_since_pipe += 1;
        if self.ticks_since_pipe > 50 {
            let mut rng = rand::rng();
            
            // Ensure gap is always within bounds
            // Gap needs to fit: gap_y (top of gap) + gap_size (gap height) must be < height
            // And gap_y must be > 0
            let min_gap_y = 2.0; // Leave some space at top
            let max_gap_y = (self.height as f32 - self.gap_size - 2.0).max(min_gap_y + 1.0); // Leave space at bottom
            
            let gap_y = if max_gap_y > min_gap_y {
                rng.random_range(min_gap_y..max_gap_y)
            } else {
                // Terminal too small, center the gap
                ((self.height as f32 - self.gap_size) / 2.0).max(0.0)
            };
            
            self.pipes.push_back((self.width as f32, gap_y, self.gap_size));
            self.ticks_since_pipe = 0;
        }

        // Pipe collision
        let bird_x = 10.0;
        let bird_r = 1.0;
        
        for p in &self.pipes {
            let px = p.0;
            let gap_y = p.1;
            let gap_size = p.2;
            
            // In pipe column?
            if bird_x + bird_r > px && bird_x < px + 2.0 {
                // Hit top or bottom pipe?
                if self.bird_y < gap_y || self.bird_y > gap_y + gap_size {
                    self.game_over = true;
                    return GameState::Running;
                }
            }
        }

        GameState::Running
    }

    fn render(&self, frame: &mut Frame) {
         let area = frame.area();

        let outer = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .split(area);

        let score_text = if self.game_over {
            format!("  🐦 GAME OVER!  Score: {}  —  Press SPACE to continue, Q to quit  ", self.score)
        } else {
            let difficulty = match self.score {
                0..=9 => "Easy",
                10..=19 => "Medium",
                20..=29 => "Hard",
                _ => "Expert",
            };
            format!("  🐦 Flappy  |  Score: {}  |  Difficulty: {}  |  SPACE to flap, Q to quit  ", self.score, difficulty)
        };
        
        let score_bar = Paragraph::new(score_text)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default());
        frame.render_widget(score_bar, outer[0]);

        let game_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Flappy Bird ")
            .title_alignment(Alignment::Center);

        let inner = game_block.inner(outer[1]);
        frame.render_widget(game_block, outer[1]);

        self.render_game(frame, inner);

        let controls = Paragraph::new("  SPACE to flap  |  Q / Esc to quit  ")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(controls, outer[2]);
    }

    fn reset(&mut self) {
        *self = FlappyGame::new();
    }
}

impl FlappyGame {
     fn render_game(&self, frame: &mut Frame, area: Rect) {
        let cell_width: u16 = 2; // Make x-coordinates wider visually

        let total_game_w = self.width * cell_width;
        let total_game_h = self.height; 
        
        let offset_x = area.x + area.width.saturating_sub(total_game_w) / 2;
        let offset_y = area.y + area.height.saturating_sub(total_game_h) / 2;

        // Render Pipes
        for p in &self.pipes {
            let px_idx = p.0 as i32;
            if px_idx >= 0 && (px_idx as u16) < self.width {
                let px = offset_x + (px_idx as u16) * cell_width;
                let gap_top = p.1 as u16;
                let gap_bottom = gap_top + p.2 as u16;
                
                for y in 0..self.height {
                     if y < gap_top || y > gap_bottom {
                         let py = offset_y + y;
                         if py_valid(py, area) && px_valid(px, cell_width, area) {
                             frame.render_widget(
                                Paragraph::new("║║").style(Style::default().fg(Color::LightGreen)),
                                Rect::new(px, py, cell_width, 1)
                             );
                         }
                     }
                }
            }
        }

        // Render Bird
        let bird_x = 10;
        let px = offset_x + bird_x * cell_width;
        let py = offset_y + self.bird_y as u16;
        
        if py_valid(py, area) && px_valid(px, cell_width, area) {
            frame.render_widget(
                Paragraph::new("🐦").style(Style::default().fg(Color::Yellow)),
                Rect::new(px, py, cell_width, 1)
            );
        }
    }
}

fn py_valid(y: u16, area: Rect) -> bool {
    y >= area.y && y < area.y + area.height
}

fn px_valid(x: u16, w: u16, area: Rect) -> bool {
    x >= area.x && x + w <= area.x + area.width
}
