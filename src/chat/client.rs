use std::io::stdout;
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use super::protocol::{Packet, read_packet, write_packet};
use super::ui::ChatUI;

pub fn run_client(host: &str, port: u16, username: &str) {
    let mut stream = match TcpStream::connect((host, port)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to connect to {}:{}: {}", host, port, e);
            return;
        }
    };

    if let Err(e) = write_packet(&mut stream, &Packet::Join { username: username.to_string() }) {
        eprintln!("Failed to send join: {}", e);
        return;
    }

    let (tx, rx) = mpsc::channel::<Packet>();
    let mut read_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to clone stream: {}", e);
            return;
        }
    };
    thread::spawn(move || {
        loop {
            match read_packet(&mut read_stream) {
                Ok(Some(packet)) => {
                    if tx.send(packet).is_err() {
                        break;
                    }
                }
                Ok(None) => {
                    let _ = tx.send(Packet::Leave { username: String::from("server") });
                    break;
                }
                Err(_) => {
                    let _ = tx.send(Packet::Leave { username: String::from("server") });
                    break;
                }
            }
        }
    });

    enable_raw_mode().expect("Failed to enable raw mode");
    execute!(stdout(), EnterAlternateScreen).expect("Failed to enter alternate screen");
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).expect("Failed to create terminal");
    terminal.clear().expect("Failed to clear terminal");

    let mut ui = ChatUI::new(username.to_string());
    let mut input_line = String::new();
    let mut running = true;

    while running {
        while let Ok(packet) = rx.try_recv() {
            ui.add_packet(packet);
        }

        while event::poll(Duration::from_millis(0)).unwrap_or(false) {
            if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
                match code {
                    KeyCode::Char(c) => {
                        input_line.push(c);
                    }
                    KeyCode::Backspace => {
                        input_line.pop();
                    }
                    KeyCode::Enter => {
                        let trimmed = input_line.trim();
                        if trimmed == "/quit" || trimmed == "/q" {
                            let _ = write_packet(&mut stream, &Packet::Leave { username: username.to_string() });
                            running = false;
                            break;
                        }
                        if !input_line.is_empty() {
                            let _ = write_packet(&mut stream, &Packet::Message {
                                username: username.to_string(),
                                content: input_line.clone(),
                            });
                            input_line.clear();
                        }
                    }
                    KeyCode::Esc => {
                        let _ = write_packet(&mut stream, &Packet::Leave { username: username.to_string() });
                        running = false;
                        break;
                    }
                    _ => {}
                }
            }
        }

        ui.set_input(input_line.clone());
        terminal.draw(|f| ui.render(f)).expect("Draw failed");
        thread::sleep(Duration::from_millis(50));
    }

    let _ = execute!(stdout(), LeaveAlternateScreen);
    let _ = disable_raw_mode();
}