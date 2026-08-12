pub mod protocol;
pub mod server;
pub mod client;
pub mod ui;

pub fn run_chat(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: sorb-chat server <port>");
        eprintln!("       sorb-chat client <host> <port> <username>");
        return;
    }

    match args[0].as_str() {
        "server" => {
            let port = args.get(1).and_then(|s| s.parse().ok()).unwrap_or(8080);
            server::run_server(port);
        }
        "client" => {
            if args.len() < 4 {
                eprintln!("Usage: sorb-chat client <host> <port> <username>");
                return;
            }
            let host = &args[1];
            let port = args[2].parse().unwrap_or(8080);
            let username = &args[3];
            client::run_client(host, port, username);
        }
        _ => {
            eprintln!("Unknown subcommand: {}", args[0]);
            eprintln!("Usage: sorb-chat server <port>");
            eprintln!("       sorb-chat client <host> <port> <username>");
        }
    }
}