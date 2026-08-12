use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use super::protocol::Packet;

pub struct ChatUI {
    username: String,
    messages: Vec<(String, String)>, // (username, content)
    input: String,
}

impl ChatUI {
    pub fn new(username: String) -> Self {
        Self {
            username,
            messages: Vec::new(),
            input: String::new(),
        }
    }

    pub fn add_packet(&mut self, packet: Packet) {
        match packet {
            Packet::Join { username } => {
                self.messages.push((String::from("system"), format!("{} joined", username)));
            }
            Packet::Message { username, content } => {
                self.messages.push((username, content));
            }
            Packet::Leave { username } => {
                self.messages.push((String::from("system"), format!("{} left", username)));
            }
        }
    }

    pub fn set_input(&mut self, input: String) {
        self.input = input;
    }

    pub fn render(&self, frame: &mut Frame) {
        let area = frame.area();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(3)])
            .split(area);

        let lines: Vec<Line> = self.messages.iter().map(|(user, msg)| {
            if user == "system" {
                Line::from(Span::styled(
                    format!("*** {} ***", msg),
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::ITALIC),
                ))
            } else {
                let color = if *user == self.username {
                    Color::Green
                } else {
                    Color::Cyan
                };
                Line::from(vec![
                    Span::styled(
                        format!("{}: ", user),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ),
                    Span::raw(msg),
                ])
            }
        }).collect();

        let messages_widget = Paragraph::new(Text::from(lines))
            .block(Block::default().title(" sorb-chat ").borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        frame.render_widget(messages_widget, chunks[0]);

        let input_widget = Paragraph::new(self.input.as_str())
            .block(Block::default().title(" message (Enter=send, Esc=quit) ").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
        frame.render_widget(input_widget, chunks[1]);
    }
}