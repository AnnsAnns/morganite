use bytes::{BufMut, BytesMut};
use log::{debug, error, info, trace, warn};
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, env::args, io};
use tokio::net::{TcpListener, TcpStream};

const LISTEN_ADDR: &str = "127.0.0.1";

struct RoutingEntry {
    info_source: String,
    destination: String,
    port: u16,
    hops: u8,
}

struct Routingtable {
    entries: Vec<RoutingEntry>,
}

impl Routingtable {
    fn new() -> Routingtable {
        Routingtable {
            entries: Vec::new(),
        }
    }

    /**
     * Adds a new entry to the routing table
     */
    fn add_entry(&mut self, entry: RoutingEntry) {
        self.entries.push(entry);
    }

    /**
     * Returns the entry with the given destination
     */
    fn get_entry(&self, destination: String) -> Option<&RoutingEntry> {
        for entry in &self.entries {
            if entry.destination == destination {
                return Some(entry);
            }
        }
        None
    }
}

fn help() {
    warn!("Usage: ./routingtable <port>");
}

#[tokio::main]
async fn main() {
    let active_connections_map: HashMap<String, TcpStream> = HashMap::new();
    let connections = Mutex::new(active_connections_map);

    simple_logger::SimpleLogger::new().env().init().unwrap();

    let port = match args().nth(1) {
        Some(port) => port,
        None => {
            help();
            return;
        }
    };
    let addr = format!("{}:{}", LISTEN_ADDR, &port);
    let listener = TcpListener::bind(&addr).await.unwrap();

    info!("Listening on: {}", &addr);
    tokio::spawn(async move {
        handle_listen(listener).await;
    });

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
                connect <destination> <port>,
                disconnect <destination>,
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
            info!("Connecting to {} on port {}", destination, port);
            connections.lock().unwrap().insert(
                destination.to_string(),
                TcpStream::connect(format!("{}:{}", destination, port))
                    .await
                    .unwrap(),
            );
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
            // @TODO: Disconnect from destination
        } else if line.starts_with("force_update") {
            info!("Forcing update");
            // @TODO: Force update
        } else {
            warn!("Unknown command: {}", line);
        }
    }
}

async fn handle_listen(listener: TcpListener) {
    info!("Listening for new connections!");

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                info!("New client connection {:?}", addr);
                tokio::spawn(async move {
                    process(socket).await;
                });
            }
            Err(e) => {
                warn!("Client connection failed = {:?}", e);
            }
        }
    }
}

async fn process(socket: TcpStream) {
    let mut msg = BytesMut::with_capacity(1024);

    loop {
        socket.readable().await.unwrap();

        match socket.try_read(&mut msg) {
            Ok(n) => {
                if n == 0 {
                    return;
                }
                debug!("Received {} bytes", n);
                debug!("Received: {:?}", msg);
                msg.clear();
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                println!("Error while reading socket: {:?}", e);
                return;
            }
        }
    }
}
