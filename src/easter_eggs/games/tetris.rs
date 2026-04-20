use rand::RngExt;
use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::easter_eggs::engine::game::{Game, GameState, Input};

const GRID_W: usize = 10;
const GRID_H: usize = 20;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tetromino {
    I, J, L, O, S, T, Z
}

impl Tetromino {
    fn color(&self) -> Color {
        match self {
            Tetromino::I => Color::Cyan,
            Tetromino::J => Color::Blue,
            Tetromino::L => Color::Rgb(255, 165, 0), // Orange
            Tetromino::O => Color::Yellow,
            Tetromino::S => Color::Green,
            Tetromino::T => Color::Magenta,
            Tetromino::Z => Color::Red,
        }
    }
    
    // Default starting orientation shapes (4x4 matrix representation to simplify rotation)
    // 0,0 is top left
    fn shape(&self) -> [[u8; 4]; 4] {
        match self {
            Tetromino::I => [
                [0,0,0,0],
                [1,1,1,1],
                [0,0,0,0],
                [0,0,0,0],
            ],
            Tetromino::J => [
                [1,0,0,0],
                [1,1,1,0],
                [0,0,0,0],
                [0,0,0,0],
            ],
            Tetromino::L => [
                [0,0,1,0],
                [1,1,1,0],
                [0,0,0,0],
                [0,0,0,0],
            ],
            Tetromino::O => [
                [0,1,1,0],
                [0,1,1,0],
                [0,0,0,0],
                [0,0,0,0],
            ],
            Tetromino::S => [
                [0,1,1,0],
                [1,1,0,0],
                [0,0,0,0],
                [0,0,0,0],
            ],
            Tetromino::T => [
                [0,1,0,0],
                [1,1,1,0],
                [0,0,0,0],
                [0,0,0,0],
            ],
            Tetromino::Z => [
                [1,1,0,0],
                [0,1,1,0],
                [0,0,0,0],
                [0,0,0,0],
            ],
        }
    }
}

fn random_piece() -> Tetromino {
    let pieces = [Tetromino::I, Tetromino::J, Tetromino::L, Tetromino::O, Tetromino::S, Tetromino::T, Tetromino::Z];
    let mut rng = rand::rng();
    pieces[rng.random_range(0..pieces.len())]
}

pub struct TetrisGame {
    grid: [[Option<Color>; GRID_W]; GRID_H],
    current: Tetromino,
    current_shape: [[u8; 4]; 4],
    current_x: i32,
    current_y: i32,
    next: Tetromino,
    score: u32,
    game_over: bool,
    tick_counter: u32,
    fall_speed: u32,
}

impl TetrisGame {
    pub fn new() -> Self {
        let current = random_piece();
        let current_shape = current.shape();
        let next = random_piece();
        Self {
            grid: [[None; GRID_W]; GRID_H],
            current,
            current_shape,
            current_x: 3,
            current_y: 0,
            next,
            score: 0,
            game_over: false,
            tick_counter: 0,
            fall_speed: 10,
        }
    }

    fn spawn(&mut self) {
        self.current = self.next;
        self.next = random_piece();
        self.current_shape = self.current.shape();
        self.current_x = 3;
        self.current_y = 0;
        if self.check_collision(self.current_x, self.current_y, &self.current_shape) {
            self.game_over = true;
        }
    }

    fn check_collision(&self, offset_x: i32, offset_y: i32, shape: &[[u8; 4]; 4]) -> bool {
        for y in 0..4 {
            for x in 0..4 {
                if shape[y][x] != 0 {
                    let map_x = offset_x + x as i32;
                    let map_y = offset_y + y as i32;
                    if map_x < 0 || map_x >= GRID_W as i32 || map_y >= GRID_H as i32 {
                        return true;
                    }
                    if map_y >= 0 && self.grid[map_y as usize][map_x as usize].is_some() {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn rotate_shape(shape: &[[u8; 4]; 4]) -> [[u8; 4]; 4] {
        let mut new_shape = [[0; 4]; 4];
        for y in 0..4 {
            for x in 0..4 {
                new_shape[x][3 - y] = shape[y][x];
            }
        }
        new_shape
    }

    fn rotate(&mut self) {
        let new_shape = Self::rotate_shape(&self.current_shape);
        if !self.check_collision(self.current_x, self.current_y, &new_shape) {
            self.current_shape = new_shape;
        }
    }

    fn lock_piece(&mut self) {
        let color = self.current.color();
        for y in 0..4 {
            for x in 0..4 {
                if self.current_shape[y][x] != 0 {
                    let map_x = self.current_x + x as i32;
                    let map_y = self.current_y + y as i32;
                    if map_y >= 0 && map_y < GRID_H as i32 && map_x >= 0 && map_x < GRID_W as i32 {
                        self.grid[map_y as usize][map_x as usize] = Some(color);
                    }
                }
            }
        }
        self.clear_lines();
        self.spawn();
    }

    fn clear_lines(&mut self) {
        let mut lines_cleared = 0;
        let mut y = GRID_H - 1;
        while y > 0 { // use while since we might modify the loop index manually on shift
            let mut full = true;
            for x in 0..GRID_W {
                if self.grid[y][x].is_none() {
                    full = false;
                    break;
                }
            }
            if full {
                lines_cleared += 1;
                // Shift everything down
                for sy in (1..=y).rev() {
                    for x in 0..GRID_W {
                        self.grid[sy][x] = self.grid[sy - 1][x];
                    }
                }
                for x in 0..GRID_W {
                    self.grid[0][x] = None;
                }
                // Don't modify y, check this same row again because the row above it just fell down into it
            } else {
                if y == 0 { break; } // prevent underflow
                y -= 1;
            }
        }

        match lines_cleared {
            1 => self.score += 100,
            2 => self.score += 300,
            3 => self.score += 500,
            4 => self.score += 800,
            _ => {}
        }
        
        // Gradually increase speed based on score (more frequent speed increases)
        let new_speed = match self.score {
            0..=499 => 10,
            500..=999 => 9,
            1000..=1499 => 8,
            1500..=1999 => 7,
            2000..=2999 => 6,
            3000..=3999 => 5,
            4000..=5999 => 4,
            6000..=8999 => 3,
            _ => 2,
        };
        self.fall_speed = new_speed;
    }

    fn drop_piece(&mut self) {
        if !self.check_collision(self.current_x, self.current_y + 1, &self.current_shape) {
            self.current_y += 1;
        } else {
            self.lock_piece();
        }
    }
}

impl Game for TetrisGame {
    fn name(&self) -> &str {
        "Tetris"
    }

    fn tick(&mut self, input: Option<Input>) -> GameState {
        if let Some(inp) = input {
            match inp {
                Input::Quit => return GameState::QuitToShell,
                Input::Action => {
                    if self.game_over {
                        return GameState::GameOver { score: self.score };
                    }
                    // Hard drop
                    while !self.check_collision(self.current_x, self.current_y + 1, &self.current_shape) {
                        self.current_y += 1;
                    }
                    self.lock_piece();
                }
                Input::Up => { // Rotate
                    if !self.game_over {
                        self.rotate();
                    }
                }
                Input::Down => { // Soft drop
                     if !self.game_over && !self.check_collision(self.current_x, self.current_y + 1, &self.current_shape) {
                         self.current_y += 1;
                     }
                }
                Input::Left => {
                    if !self.game_over && !self.check_collision(self.current_x - 1, self.current_y, &self.current_shape) {
                        self.current_x -= 1;
                    }
                }
                Input::Right => {
                     if !self.game_over && !self.check_collision(self.current_x + 1, self.current_y, &self.current_shape) {
                        self.current_x += 1;
                    }
                }
            }
        }

        if self.game_over {
            return GameState::Running;
        }

        self.tick_counter += 1;
        if self.tick_counter >= self.fall_speed {
            self.drop_piece();
            self.tick_counter = 0;
        }

        GameState::Running
    }

    fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let outer = Layout::vertical([Constraint::Length(3), Constraint::Min(1), Constraint::Length(3)]).split(area);

        let score_text = if self.game_over {
            format!("  You're all bricked up....wait no-!  Score: {}  —  Press SPACE to continue, Q to quit  ", self.score)
        } else {
            format!("  Tetris  |  Score: {}  |  Speed: {}  |  Arrows to move/rotate, SPACE to drop  ", self.score, 11 - self.fall_speed)
        };
        
        let score_bar = Paragraph::new(score_text).style(Style::default().fg(Color::White).bg(Color::DarkGray).add_modifier(Modifier::BOLD)).alignment(Alignment::Center).block(Block::default());
        frame.render_widget(score_bar, outer[0]);

        let game_block = Block::default().borders(Borders::ALL).border_style(Style::default().fg(Color::DarkGray));
        let inner = game_block.inner(outer[1]);
        frame.render_widget(game_block, outer[1]);

        let cell_width = 2;
        let total_w = (GRID_W as u16) * cell_width;
        let total_h = GRID_H as u16;
        
        // Calculate offset to center the game area, leaving room for next piece preview on the right
        let preview_width = 12; // Width for the "Next" preview box
        let game_area_width = total_w + preview_width + 4; // game + spacing + preview
        let offset_x = inner.x + inner.width.saturating_sub(game_area_width) / 2;
        let offset_y = inner.y + inner.height.saturating_sub(total_h) / 2;

        // Draw well boundaries
        for y in 0..GRID_H as u16 {
             frame.render_widget(Paragraph::new("│").style(Style::default().fg(Color::DarkGray)), Rect::new(offset_x - 1, offset_y + y, 1, 1));
             frame.render_widget(Paragraph::new("│").style(Style::default().fg(Color::DarkGray)), Rect::new(offset_x + total_w, offset_y + y, 1, 1));
        }
        let bottom_line = "└".to_string() + &"─".repeat(total_w as usize) + "┘";
        frame.render_widget(Paragraph::new(bottom_line).style(Style::default().fg(Color::DarkGray)), Rect::new(offset_x - 1, offset_y + total_h, total_w + 2, 1));

        // Draw locked grid
        for y in 0..GRID_H {
            for x in 0..GRID_W {
                if let Some(color) = self.grid[y][x] {
                    let px = offset_x + (x as u16) * cell_width;
                    let py = offset_y + (y as u16);
                    frame.render_widget(Paragraph::new("██").style(Style::default().fg(color)), Rect::new(px, py, cell_width, 1));
                }
            }
        }

        // Draw current piece
        let color = self.current.color();
        for y in 0..4 {
            for x in 0..4 {
                if self.current_shape[y][x] != 0 {
                    let abs_x = self.current_x + x as i32;
                    let abs_y = self.current_y + y as i32;
                    if abs_y >= 0 && abs_y < GRID_H as i32 && abs_x >= 0 && abs_x < GRID_W as i32 {
                         let px = offset_x + (abs_x as u16) * cell_width;
                         let py = offset_y + (abs_y as u16);
                         frame.render_widget(Paragraph::new("██").style(Style::default().fg(color)), Rect::new(px, py, cell_width, 1));
                    }
                }
            }
        }

        // Draw "Next" piece preview box
        let preview_x = offset_x + total_w + 3;
        let preview_y = offset_y + 2;
        let preview_box = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Gray))
            .title("Next");
        let preview_area = Rect::new(preview_x, preview_y, 10, 6);
        frame.render_widget(preview_box, preview_area);

        // Draw next piece inside the preview box
        let next_shape = self.next.shape();
        let next_color = self.next.color();
        for y in 0..4 {
            for x in 0..4 {
                if next_shape[y][x] != 0 {
                    let px = preview_x + 2 + (x as u16) * cell_width;
                    let py = preview_y + 2 + (y as u16);
                    frame.render_widget(Paragraph::new("██").style(Style::default().fg(next_color)), Rect::new(px, py, cell_width, 1));
                }
            }
        }
    }

    fn reset(&mut self) {
        *self = TetrisGame::new();
    }
}
