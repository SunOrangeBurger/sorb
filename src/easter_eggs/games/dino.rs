use std::collections::VecDeque;

use rand::RngExt;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::easter_eggs::engine::game::{Game, GameState, Input};

const GRAVITY: f32 = 0.8;
const JUMP_FORCE: f32 = 3.2;

#[derive(Debug, Clone)]
enum ObstacleType {
    Cactus { width: f32, height: f32 },
    Pterodactyl { y_offset: f32 },
}

#[derive(Debug, Clone)]
struct Obstacle {
    x: f32,
    kind: ObstacleType,
}

pub struct DinoGame {
    /// Elevation above ground (0.0 = on ground)
    dino_y: f32,
    /// Vertical velocity
    velocity_y: f32,
    /// Obstacles
    obstacles: VecDeque<Obstacle>,
    score: u32,
    game_over: bool,
    ticks_since_obstacle: u32,
    width: u16,
    height: u16,
    speed: f32,
    duck_timer: u32,
}

impl DinoGame {
    pub fn new() -> Self {
        let mut width = 40;
        let mut height = 20;
        if let Ok((term_w, term_h)) = crossterm::terminal::size() {
            width = (term_w.saturating_sub(2) / 2).max(10);
            height = term_h.saturating_sub(8).max(10);
        }

        Self {
            dino_y: 0.0,
            velocity_y: 0.0,
            obstacles: VecDeque::new(),
            score: 0,
            game_over: false,
            ticks_since_obstacle: 0,
            width,
            height,
            speed: 1.5,
            duck_timer: 0,
        }
    }
}

impl Game for DinoGame {
    fn name(&self) -> &str {
        "Dino"
    }

    fn tick(&mut self, input: Option<Input>) -> GameState {
        // Update dimensions
        if let Ok((term_w, term_h)) = crossterm::terminal::size() {
            self.width = (term_w.saturating_sub(2) / 2).max(10);
            self.height = term_h.saturating_sub(8).max(10);
        }

        if let Some(inp) = input {
            match inp {
                Input::Quit => return GameState::QuitToShell,
                Input::Action | Input::Up => {
                    if self.game_over {
                        return GameState::GameOver { score: self.score };
                    } else if self.dino_y == 0.0 {
                        // Jump height scales dynamically with current speed
                        let dynamic_jump_force = JUMP_FORCE + ((self.speed - 1.5) * 0.8);
                        self.velocity_y = dynamic_jump_force;
                        self.duck_timer = 0;
                    }
                }
                Input::Down => {
                    if self.game_over {
                        return GameState::GameOver { score: self.score };
                    }
                    if self.dino_y > 0.0 {
                        // fast fall
                        self.velocity_y -= GRAVITY * 2.0;
                    } else {
                        // duck
                        self.duck_timer = 10;
                    }
                }
                _ => {}
            }
        }

        if self.game_over {
            return GameState::Running;
        }

        if self.duck_timer > 0 {
            self.duck_timer -= 1;
        }

        // Apply physics
        if self.dino_y > 0.0 || self.velocity_y > 0.0 || self.velocity_y < 0.0 {
            self.velocity_y -= GRAVITY;
            self.dino_y += self.velocity_y;
        }

        // Ground collision
        if self.dino_y <= 0.0 {
            self.dino_y = 0.0;
            self.velocity_y = 0.0;
        }

        // Move obstacles
        for ob in self.obstacles.iter_mut() {
            ob.x -= self.speed;
        }

        // Remove off-screen obstacles
        if let Some(front) = self.obstacles.front() {
            if front.x < -4.0 {
                self.obstacles.pop_front();
            }
        }

        // Spawn obstacles
        self.ticks_since_obstacle += 1;
        let spawn_threshold = (30.0 / self.speed) as u32;
        
        if self.ticks_since_obstacle > spawn_threshold {
            let mut rng = rand::rng();
            if rng.random_range(0..100) < f32::min(8.0 + self.speed, 20.0) as i32 { 
                if self.score > 150 && rng.random_range(0..100) < 30 {
                    let y_offset = rng.random_range(1.0..4.0);
                    self.obstacles.push_back(Obstacle {
                        x: self.width as f32,
                        kind: ObstacleType::Pterodactyl { y_offset },
                    });
                } else {
                    let height = rng.random_range(1.0..4.0);
                    let width = rng.random_range(1.0..2.5);
                    self.obstacles.push_back(Obstacle {
                        x: self.width as f32,
                        kind: ObstacleType::Cactus { width, height },
                    });
                }
                self.ticks_since_obstacle = 0;
            }
        }

        // Collision detection
        let dino_x = 5.0;
        let is_ducking = self.duck_timer > 0 && self.dino_y == 0.0;
        
        // Logical bounds for AABB
        let (dino_w, dino_h) = if is_ducking {
            (3.0, 1.5)
        } else {
            (2.5, 2.5) // Slightly forgiving hitbox
        };
        
        let dino_y_bottom = self.dino_y;
        let dino_y_top = self.dino_y + dino_h;

        for ob in &self.obstacles {
            let (ob_x, ob_w, ob_y_bottom, ob_y_top) = match ob.kind {
                ObstacleType::Cactus { width, height } => {
                    (ob.x, width, 0.0, height)
                }
                ObstacleType::Pterodactyl { y_offset } => {
                    (ob.x, 1.5, y_offset, y_offset + 1.0)
                }
            };

            // Calculate overlap (AABB)
            let x_overlap = dino_x < ob_x + ob_w && dino_x + dino_w > ob_x;
            let y_overlap = dino_y_bottom < ob_y_top && dino_y_top > ob_y_bottom;

            if x_overlap && y_overlap {
                self.game_over = true;
                return GameState::Running;
            }
        }

        self.score += 1;
        // Gradually increase speed
        if self.score % 100 == 0 {
            self.speed += 0.1;
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
            format!(" The asteroid done got yo ahh !  Score: {}  —  Press SPACE to continue, Q to quit  ", self.score)
        } else {
            let speed_level = ((self.speed - 1.5) / 0.1) as u32 + 1;
            format!(" Dino  |  Score: {}  |  Speed: {}   ", self.score, speed_level)
        };
        
        let score_bar = Paragraph::new(score_text)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default());
        frame.render_widget(score_bar, outer[0]);

        let game_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(" Dino ")
            .title_alignment(Alignment::Center);

        let inner = game_block.inner(outer[1]);
        frame.render_widget(game_block, outer[1]);

        self.render_game(frame, inner);

        let controls = Paragraph::new("  SPACE/↑ to jump  |  ↓ to duck  |  Q / Esc to quit  ")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(controls, outer[2]);
    }

    fn reset(&mut self) {
        *self = DinoGame::new();
    }
}

impl DinoGame {
    fn render_game(&self, frame: &mut Frame, area: Rect) {
        let cell_width: u16 = 2;

        let total_game_w = self.width * cell_width;
        let offset_x = area.x + area.width.saturating_sub(total_game_w) / 2;

        // The ground is established at the very bottom of the viewable area allowing Dino space 
        let ground_y = area.y + area.height.saturating_sub(2);
        
        if ground_y < area.y + area.height {
            let mut ground_line = String::new();
            for _ in 0..(self.width * cell_width) {
               ground_line.push('═');
            }
            
            draw_line(frame, &ground_line, offset_x, ground_y, Color::Gray, area);
        }

        // Render Obstacles
        for ob in &self.obstacles {
            let px_offset = (ob.x * cell_width as f32) as i32;
            if px_offset < 0 { continue; }
            let px = offset_x.saturating_add(px_offset as u16);

            match &ob.kind {
                ObstacleType::Cactus { width, height } => {
                    let w_chars = (*width * cell_width as f32) as usize;
                    let text = "█".repeat(w_chars);
                    let h_cells = height.ceil() as u16;
                    let ob_draw_y = ground_y.saturating_sub(1);
                    
                    for i in 0..h_cells {
                        let py = ob_draw_y.saturating_sub(i);
                        draw_line(frame, &text, px, py, Color::Green, area);
                    }
                }
                ObstacleType::Pterodactyl { y_offset } => {
                    let py = ground_y.saturating_sub(1).saturating_sub(*y_offset as u16);
                    let wing = if (self.score / 5) % 2 == 0 {
                        "\\v/"
                    } else {
                        "--v--"
                    };
                    draw_line(frame, wing, px, py, Color::Red, area);
                }
            }
        }

        // Render Dino
        let dino_x = 5;
        let px = offset_x + dino_x * cell_width;
        let is_ducking = self.duck_timer > 0 && self.dino_y == 0.0;
        
        let dino_draw_y = ground_y.saturating_sub(1).saturating_sub(self.dino_y as u16);

        let sprite = if is_ducking {
            vec![
                "   █▀▄",
                "▀▀███▀",
                "  ▀ ▀ ",
            ]
        } else {
            vec![
                "  █▀▄",
                "█▀█▀ ",
                "▀ ▀  ",
            ]
        };

        for (i, line) in sprite.iter().enumerate() {
            let py = dino_draw_y.saturating_sub((sprite.len() - 1 - i) as u16);
            draw_line(frame, *line, px, py, Color::LightMagenta, area);
        }
    }
}

// Helper to draw clipping lines within bounds
fn draw_line(frame: &mut Frame, text: &str, x: u16, y: u16, color: Color, area: Rect) {
    if y >= area.y && y < area.y + area.height && x < area.x + area.width {
        let max_w = (area.x + area.width).saturating_sub(x);
        let text_w = text.chars().count() as u16;
        let render_w = text_w.min(max_w);
        if render_w > 0 {
            // Trim the text to fit rendering width safely
            let slice: String = text.chars().take(render_w as usize).collect();
            frame.render_widget(
                Paragraph::new(slice).style(Style::default().fg(color)),
                Rect::new(x, y, render_w, 1)
            );
        }
    }
}
