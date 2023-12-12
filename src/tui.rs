use std::io;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::Morganite;

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
                info!(
                    "Available commands:
                exit,
                help,
                connect <IP> <port>,
                disconnect <name>,
                show_routingtable,
                force_update
            "
                );
            } else if line.starts_with("dbg1") {
                self.connect("127.0.0.1", "12345").await;
            } else if line.starts_with("dbg2") {
                self.connect("127.0.0.1", "12346").await;
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

                self.connect(destination, port).await;
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
            } else if line.starts_with("show_routingtable") || line.starts_with("list_routingtable") {
                self.morganite.lock().await.print_routingtable().await;
            } else if line.starts_with("force_update") {
                info!("Forcing update");
            } else {
                warn!("Unknown command: {}", line);
            }
        }
    }

    pub async fn connect(&mut self, destination: &str, port: &str) {
        info!("Connecting to {} on port {}", destination, port);
        self
            .morganite
            .lock()
            .await
            .connect_new(
                destination.to_string(),
                port.to_string(),
                "ZZZ".to_string(),
            )
            .await;
    }
}
