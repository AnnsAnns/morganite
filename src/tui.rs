use std::{io, sync::{Mutex, Arc}};

use crate::Morganite;

use log::{debug, error, info, trace, warn};

pub struct Tui {
    morganite: Arc<Mutex<Morganite>>,
}

impl Tui {
    pub fn new(morganite: Arc<Mutex<Morganite>>) -> Tui {
        Tui { morganite }
    }

    pub fn handle_console(&self) {
        let stdin = io::stdin();
        let mut line = String::new();
        loop {
            stdin.read_line(&mut line).unwrap();
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
                    Some(name) => name,
                    None => {
                        warn!("Missing name");
                        continue;
                    }
                };
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
                self.morganite.lock().unwrap().print_routingtable();
            } else if line.starts_with("force_update") {
                info!("Forcing update");
            } else {
                warn!("Unknown command: {}", line);
            }
        }
    }
}
