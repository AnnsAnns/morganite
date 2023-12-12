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

    pub async fn handle_console(&self) {
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
                connect <IP> <port> <name>,
                disconnect <name>,
                show_routingtable,
                force_update
            "
                );
            } else if line.starts_with("connect") {
                let mut args = line.split_whitespace();
                args.next();
                let _destination = match args.next() {
                    Some(destination) => destination,
                    None => {
                        warn!("Missing destination");
                        continue;
                    }
                };
                let _port = match args.next() {
                    Some(port) => port,
                    None => {
                        warn!("Missing port");
                        continue;
                    }
                };
                let _target_name = match args.next() {
                    Some(name) => name,
                    None => {
                        warn!("Missing name");
                        continue;
                    }
                };

                info!("Connecting to {} on port {}", _destination, _port);
                let _x = self
                    .morganite
                    .lock()
                    .await
                    .connect_new(
                        _destination.to_string(),
                        _port.to_string(),
                        _target_name.to_string(),
                    )
                    .await;
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
            } else if line.starts_with("show_routingtable") {
                self.morganite.lock().await.print_routingtable().await;
            } else if line.starts_with("force_update") {
                info!("Forcing update");
            } else {
                warn!("Unknown command: {}", line);
            }
        }
    }
}
