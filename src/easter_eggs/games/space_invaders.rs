use rand::RngExt;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::easter_eggs::engine::game::{Game, GameState, Input};

const SHIP_Y: f32 = 18.0;

struct Alien {
    x: f32,
    y: f32,
    exploding: bool,
    explosion_timer: u32,
}

struct Laser {
    x: f32,
    y: f32,
    vx: f32, // velocity x for relative motion
}

pub struct SpaceInvadersGame {
    ship_x: f32,
    ship_vx: f32, // ship velocity for relative motion
    ship_cooldown: u32,
    player_lasers: Vec<Laser>,
    alien_lasers: Vec<(f32, f32)>,
    aliens: Vec<Alien>,
    alien_dir: f32,
    alien_speed: f32,
    ticks_since_alien_move: u32,
    score: u32,
    wave: u32,
    lives: u32,
    game_over: bool,
    width: u16,
    height: u16,
    consecutive_hits: u32,
    laser_active: bool,
    laser_timer: u32,
    shots_fired: u32, // Track total shots to detect misses
    auto_shoot: bool, // Enable auto-shooting mode
}

impl SpaceInvadersGame {
    pub fn new() -> Self {
        let mut sim = Self {
            ship_x: 20.0,
            ship_vx: 0.0,
            ship_cooldown: 0,
            player_lasers: Vec::new(),
            alien_lasers: Vec::new(),
            aliens: Vec::new(),
            alien_dir: 1.0,
            alien_speed: 15.0, // Ticks between move
            ticks_since_alien_move: 0,
            score: 0,
            wave: 1,
            lives: 3,
            game_over: false,
            width: 40,
            height: 20,
            consecutive_hits: 0,
            laser_active: false,
            laser_timer: 0,
            shots_fired: 0,
            auto_shoot: true, // Enable auto-shoot by default
        };
        sim.init_wave();
        sim
    }

    fn init_wave(&mut self) {
        self.aliens.clear();
        self.player_lasers.clear();
        self.alien_lasers.clear();
        
        let cols = 8;
        let rows = 4;
        for row in 0..rows {
            for col in 0..cols {
                self.aliens.push(Alien {
                    x: 5.0 + (col as f32) * 3.0,
                    y: 2.0 + (row as f32) * 2.0,
                    exploding: false,
                    explosion_timer: 0,
                });
            }
        }
        self.alien_dir = 1.0;
        self.alien_speed = (15.0 - (self.wave as f32)).max(2.0); // gets faster each wave
    }
}

impl Game for SpaceInvadersGame {
    fn name(&self) -> &str {
        "Space Invaders"
    }

    fn tick(&mut self, input: Option<Input>) -> GameState {
        let mut shoot = false;
        let mut moving = false;
        
        if let Some(inp) = input {
            match inp {
                Input::Quit => return GameState::QuitToShell,
                Input::Action => {
                    if self.game_over {
                        return GameState::GameOver { score: self.score };
                    }
                    shoot = true;
                }
                Input::Left => {
                    let new_x = (self.ship_x - 1.0).max(0.0);
                    self.ship_vx = new_x - self.ship_x;
                    self.ship_x = new_x;
                    moving = true;
                }
                Input::Right => {
                    let new_x = (self.ship_x + 1.0).min(self.width as f32 - 2.0);
                    self.ship_vx = new_x - self.ship_x;
                    self.ship_x = new_x;
                    moving = true;
                }
                Input::Up => {
                    // Use Up arrow as alternative shoot button
                    if !self.game_over {
                        shoot = true;
                    }
                }
                _ => {}
            }
        }
        
        if !moving {
            self.ship_vx = 0.0;
        }

        // Auto-shoot mode: if auto_shoot is enabled, shoot continuously
        if self.auto_shoot && self.ship_cooldown == 0 && !self.laser_active {
            shoot = true;
        }

        // Handle shooting
        if shoot && self.ship_cooldown == 0 && !self.laser_active {
            self.player_lasers.push(Laser {
                x: self.ship_x + 0.5,
                y: SHIP_Y - 1.0,
                vx: self.ship_vx, // Inherit ship velocity
            });
            self.ship_cooldown = 5;
            self.shots_fired += 1;
        }

        if self.game_over {
            return GameState::Running;
        }

        if self.ship_cooldown > 0 {
            self.ship_cooldown -= 1;
        }

        // Handle space laser
        if self.laser_active {
            self.laser_timer += 1;
            if self.laser_timer >= 60 { // 1 second at 60ms ticks
                self.laser_active = false;
                self.laser_timer = 0;
            }
            
            // Space laser hits aliens
            let laser_x = self.ship_x + 0.5;
            for alien in self.aliens.iter_mut() {
                if !alien.exploding && laser_x >= alien.x && laser_x <= alien.x + 2.0 {
                    alien.exploding = true;
                    alien.explosion_timer = 0;
                    self.score += 10 * self.wave;
                }
            }
        }

        // Move player lasers with relative motion
        for pl in self.player_lasers.iter_mut() {
            pl.y -= 1.0;
            pl.x += pl.vx * 0.5; // Apply horizontal velocity
        }
        self.player_lasers.retain(|pl| pl.y >= 0.0 && pl.x >= 0.0 && pl.x < self.width as f32);

        // Move alien lasers
        for al in self.alien_lasers.iter_mut() {
            al.1 += 0.5;
            // Alien laser hit player?
            if al.1 >= SHIP_Y && al.1 <= SHIP_Y + 1.0 && al.0 >= self.ship_x && al.0 <= self.ship_x + 2.0 {
                self.lives = self.lives.saturating_sub(1);
                al.1 = self.height as f32 + 10.0; // move off-screen
                if self.lives == 0 {
                    self.game_over = true;
                }
            }
        }
        self.alien_lasers.retain(|al| al.1 <= self.height as f32);

        // Bullet interception - player lasers can hit alien lasers
        let mut hit_alien_laser_indices = vec![];
        let mut hit_player_laser_indices = vec![];
        for (pl_idx, pl) in self.player_lasers.iter().enumerate() {
            for (al_idx, al) in self.alien_lasers.iter().enumerate() {
                let dx = (pl.x - al.0).abs();
                let dy = (pl.y - al.1).abs();
                if dx < 0.5 && dy < 0.5 {
                    hit_alien_laser_indices.push(al_idx);
                    hit_player_laser_indices.push(pl_idx);
                }
            }
        }
        
        hit_alien_laser_indices.sort_unstable();
        hit_alien_laser_indices.dedup();
        for &idx in hit_alien_laser_indices.iter().rev() {
            self.alien_lasers.remove(idx);
        }
        hit_player_laser_indices.sort_unstable();
        hit_player_laser_indices.dedup();
        for &idx in hit_player_laser_indices.iter().rev() {
            self.player_lasers.remove(idx);
        }

        // Check player laser hits alien
        let mut hit_alien_indices = vec![];
        let mut hit_laser_indices = vec![];
        for (l_idx, l) in self.player_lasers.iter().enumerate() {
            let mut laser_hit_something = false;
            for (a_idx, a) in self.aliens.iter().enumerate() {
                if !a.exploding && l.x >= a.x && l.x <= a.x + 2.0 && l.y >= a.y && l.y <= a.y + 1.0 {
                    hit_alien_indices.push(a_idx);
                    hit_laser_indices.push(l_idx);
                    laser_hit_something = true;
                }
            }
            
            // Check if laser went off screen without hitting anything
            if !laser_hit_something && l.y < 0.0 {
                // This laser missed - reset combo
                self.consecutive_hits = 0;
            }
        }

        if !hit_alien_indices.is_empty() {
            self.consecutive_hits += hit_alien_indices.len() as u32;
            
            // Activate space laser after 10 consecutive hits
            if self.consecutive_hits >= 10 && !self.laser_active {
                self.laser_active = true;
                self.laser_timer = 0;
                self.consecutive_hits = 0;
            }
        }

        hit_alien_indices.sort_unstable();
        hit_alien_indices.dedup();
        for &idx in hit_alien_indices.iter() {
            if idx < self.aliens.len() {
                self.aliens[idx].exploding = true;
                self.aliens[idx].explosion_timer = 0;
                self.score += 10 * self.wave;
            }
        }
        hit_laser_indices.sort_unstable();
        hit_laser_indices.dedup();
        for &idx in hit_laser_indices.iter().rev() {
            self.player_lasers.remove(idx);
        }

        // Update explosion animations and remove finished explosions
        self.aliens.retain_mut(|a| {
            if a.exploding {
                a.explosion_timer += 1;
                a.explosion_timer < 10 // Keep for 10 ticks
            } else {
                true
            }
        });

        if self.aliens.iter().all(|a| a.exploding) && self.aliens.is_empty() {
            self.wave += 1;
            self.init_wave();
            return GameState::Running;
        }

        // Form alien behavior
        self.ticks_since_alien_move += 1;
        if (self.ticks_since_alien_move as f32) >= self.alien_speed {
            self.ticks_since_alien_move = 0;
            
            let mut hit_wall = false;
            for a in &self.aliens {
                if !a.exploding && ((a.x + self.alien_dir <= 0.0) || (a.x + 2.0 + self.alien_dir >= self.width as f32)) {
                    hit_wall = true;
                    break;
                }
            }

            if hit_wall {
                self.alien_dir *= -1.0;
                for a in self.aliens.iter_mut() {
                    if !a.exploding {
                        a.y += 1.0;
                        if a.y + 1.0 >= SHIP_Y {
                            self.lives = 0;
                            self.game_over = true; // Invaders reached bottom
                        }
                    }
                }
            } else {
                for a in self.aliens.iter_mut() {
                    if !a.exploding {
                        a.x += self.alien_dir;
                    }
                }
            }
            
            // Random alien shots
            let mut rng = rand::rng();
            let active_aliens: Vec<&Alien> = self.aliens.iter().filter(|a| !a.exploding).collect();
            if !active_aliens.is_empty() && rng.random_range(0..100) < 15 + self.wave * 2 {
                let r_alien = active_aliens[rng.random_range(0..active_aliens.len())];
                self.alien_lasers.push((r_alien.x + 1.0, r_alien.y + 1.0));
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
            format!("  👾 GAME OVER!  Score: {} (Wave: {})  —  Press SPACE to continue, Q to quit  ", self.score, self.wave)
        } else {
            let difficulty = match self.wave {
                1..=2 => "Easy",
                3..=4 => "Medium",
                5..=7 => "Hard",
                _ => "Expert",
            };
            let combo_text = if self.laser_active {
                format!(" ⚡LASER ACTIVE⚡ ")
            } else if self.consecutive_hits > 0 {
                format!(" Combo: {}/10 ", self.consecutive_hits)
            } else {
                String::new()
            };
            format!("  👾 Invaders  |  Lives: {}  |  Score: {}  |  Wave: {} ({}){}  |  ← → move, ↑/SPACE shoot  ", self.lives, self.score, self.wave, difficulty, combo_text)
        };
        
        let score_bar = Paragraph::new(score_text)
            .style(Style::default().fg(Color::White).bg(Color::DarkGray).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default());
        frame.render_widget(score_bar, outer[0]);

        let game_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray));

        let inner = game_block.inner(outer[1]);
        frame.render_widget(game_block, outer[1]);

        let cell_width = 2;
        let total_w = self.width * cell_width;
        let offset_x = inner.x + inner.width.saturating_sub(total_w) / 2;
        let offset_y = inner.y;

        // Player ship
        let px = offset_x + (self.ship_x as u16) * cell_width;
        let py = offset_y + SHIP_Y as u16;
        frame.render_widget(Paragraph::new("▀▄▀").style(Style::default().fg(Color::Cyan)), Rect::new(px, py, 3, 1));
        
        // Space laser beam
        if self.laser_active {
            let laser_x = offset_x + ((self.ship_x + 0.5) as u16) * cell_width;
            for y in 0..SHIP_Y as u16 {
                let ly = offset_y + y;
                let beam_char = if (self.laser_timer / 2) % 2 == 0 { "║" } else { "│" };
                frame.render_widget(
                    Paragraph::new(beam_char).style(Style::default().fg(Color::LightYellow).add_modifier(Modifier::BOLD)),
                    Rect::new(laser_x, ly, 1, 1)
                );
            }
        }
        
        // Aliens
        for a in &self.aliens {
            let ax = offset_x + (a.x as u16) * cell_width;
            let ay = offset_y + a.y as u16;
            
            if a.exploding {
                // Explosion animation
                let explosion_frame = match a.explosion_timer {
                    0..=2 => "***",
                    3..=5 => "*+*",
                    6..=8 => " + ",
                    _ => "   ",
                };
                frame.render_widget(
                    Paragraph::new(explosion_frame).style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Rect::new(ax, ay, 3, 1)
                );
            } else {
                frame.render_widget(
                    Paragraph::new("M-M").style(Style::default().fg(Color::Green)),
                    Rect::new(ax, ay, 3, 1)
                );
            }
        }

        // Player lasers
        for l in &self.player_lasers {
             let lx = offset_x + (l.x as u16) * cell_width;
             let ly = offset_y + l.y as u16;
             if ly >= inner.y {
                 frame.render_widget(Paragraph::new("|").style(Style::default().fg(Color::Yellow)), Rect::new(lx, ly, 1, 1));
             }
        }

        // Alien lasers
        for l in &self.alien_lasers {
             let lx = offset_x + (l.0 as u16) * cell_width;
             let ly = offset_y + l.1 as u16;
             if ly < inner.y + inner.height {
                 frame.render_widget(Paragraph::new("v").style(Style::default().fg(Color::Red)), Rect::new(lx, ly, 1, 1));
             }
        }
    }

    fn reset(&mut self) {
        *self = SpaceInvadersGame::new();
    }
}
