use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::Morganite;

use colored::*;
use log::{debug, info, warn};

pub struct Tui {
    morganite: Arc<Mutex<Morganite>>,
}

impl Tui {
    pub fn new(morganite: Arc<Mutex<Morganite>>) -> Tui {
        Tui { morganite }
    }

    pub async fn handle_console(&mut self) {
        loop {
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();
            let line = line.trim();
            debug!("Command entered: {}", line);

            if line.starts_with("exit") {
                return;
            } else if line.starts_with("help") {
                println!(
                    "{}\n{}",
                    "\nAvailable commands:".italic().underline(),
                    "        exit - Exit the program,
        help - Show this help,
        connect <IP> <port> <name> - Connect to a node,
        disconnect <name> - Disconnect from a node,
        show_routingtable - Show the routing table,
        force_update - Force an update of the routing table,
        send <name> <message> - Send a message to a node,"
                );
            } else if line.starts_with("dbg1") {
                self.connect("127.0.0.1", "12345", "AAA").await;
            } else if line.starts_with("dbg2") {
                self.connect("127.0.0.1", "12346", "BBB").await;
            } else if line.starts_with("dbg3") {
                self.connect("127.0.0.1", "12347", "CCC").await;
            } else if line.starts_with("connect") {
                let mut args = line.split_whitespace();
                args.next();
                let destination = match args.next() {
                    Some(destination) => destination,
                    None => {
                        warn!("Missing destination");
                        continue;
                    }
                };
                let port = match args.next() {
                    Some(port) => port,
                    None => {
                        warn!("Missing port");
                        continue;
                    }
                };
                let target_name = match args.next() {
                    Some(target_name) => target_name,
                    None => {
                        warn!("Missing target name");
                        continue;
                    }
                };

                self.connect(destination, port, target_name).await;
            } else if line.starts_with("disconnect") {
                let mut args = line.split_whitespace();
                args.next();
                let destination = match args.next() {
                    Some(destination) => destination,
                    None => {
                        warn!("Missing destination");
                        continue;
                    }
                };
                info!("Disconnecting from {}", destination);
            } else if line.starts_with("show_routingtable") || line.starts_with("list_routingtable")
            {
                self.morganite.lock().await.print_routingtable().await;
            } else if line.starts_with("force_update") {
                info!("Forcing update");
                self.morganite.lock().await.broadcast_routingtable().await;
            } else if line.starts_with("send") {
                let mut args = line.split_whitespace();
                args.next();
                let destination = match args.next() {
                    Some(destination) => destination,
                    None => {
                        warn!("Missing destination");
                        continue;
                    }
                };

                // Collect the rest of the line as message
                let mut message = args.collect::<Vec<&str>>().join(" ");

                // The message can only be 320 characters long
                if message.len() > 320 {
                    warn!("Message is too long, truncating to 320 characters");
                    message.truncate(320);
                }

                debug!("Sending message to {}: {}", destination, message);

                println!(
                    "📤 {}",
                    format!("@You to @{}:\n{}", destination, message)
                        .trim()
                        .bright_white()
                        .on_magenta()
                        .italic()
                );

                self.morganite
                    .lock()
                    .await
                    .send_message(destination.to_string(), message.to_string())
                    .await;
            } else {
                warn!("Unknown command: {}", line);
            }
        }
    }

    pub async fn connect(&mut self, destination: &str, port: &str, target_name: &str) {
        info!("Connecting to {} on port {}", destination, port);
        self.morganite
            .lock()
            .await
            .connect_new(
                destination.to_string(),
                port.to_string(),
                target_name.to_string(),
            )
            .await;
    }
}
