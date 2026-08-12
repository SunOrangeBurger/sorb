use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;
use super::protocol::{read_packet, write_packet};

fn get_local_ip() -> Option<String> {
    if let Ok(socket) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if socket.connect("8.8.8.8:80").is_ok() {
            if let Ok(addr) = socket.local_addr() {
                return Some(addr.ip().to_string());
            }
        }
    }
    None
}

pub fn run_server(port: u16) {
    let listener = TcpListener::bind(("0.0.0.0", port)).expect("Failed to bind server");
    println!("sorb-chat server listening on 0.0.0.0:{}", port);
    
    if let Some(ip) = get_local_ip() {
        println!("Other devices can connect with: sorb-chat client {} {} <name>", ip, port);
    } else {
        println!("Other devices can connect with: sorb-chat client <this-ip> {} <name>", port);
    }
    println!("Close this terminal tab to stop the server.");
    
    let clients: Arc<Mutex<Vec<TcpStream>>> = Arc::new(Mutex::new(Vec::new()));
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Ok(addr) = stream.peer_addr() {
                    println!("Client connected: {}", addr);
                }
                let clients = Arc::clone(&clients);
                
                if let Ok(stream_clone) = stream.try_clone() {
                    clients.lock().unwrap().push(stream_clone);
                }
                
                thread::spawn(move || {
                    handle_client(stream, clients);
                });
            }
            Err(e) => {
                eprintln!("Accept error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream, clients: Arc<Mutex<Vec<TcpStream>>>) {
    loop {
        match read_packet(&mut stream) {
            Ok(Some(packet)) => {
                let mut list = clients.lock().unwrap();
                let mut dead = Vec::new();
                for (i, client) in list.iter_mut().enumerate() {
                    if let Err(e) = write_packet(client, &packet) {
                        eprintln!("Broadcast error to client {}: {}", i, e);
                        dead.push(i);
                    }
                }
                for idx in dead.into_iter().rev() {
                    list.remove(idx);
                }
            }
            Ok(None) => {
                println!("Client disconnected");
                break;
            }
            Err(e) => {
                eprintln!("Client read error: {}", e);
                break;
            }
        }
    }
}