use bytes::{BufMut, BytesMut};
use log::{debug, error, info, trace, warn};
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, env::args, io};
use tokio::net::{TcpListener, TcpStream};

mod header;
mod routing;

use header::BaseHeader;
use routing::{RoutingEntry, Routingtable};

const LISTEN_ADDR: &str = "127.0.0.1";

fn help() {
    warn!("Usage: ./routingtable <port> <name (max 3 chars)>");
}

#[tokio::main]
async fn main() {
    // Create connections map and wrap it in a Mutex
    let active_connections_map: HashMap<String, TcpStream> = HashMap::new();
    let connections = Mutex::new(active_connections_map);

    // Create routing table and wrap it in an Arc<Mutex>
    let routingtable = Arc::new(Mutex::new(Routingtable::new()));

    // Init logger
    simple_logger::SimpleLogger::new().env().init().unwrap();

    // Parse Args
    let port = match args().nth(1) {
        Some(port) => port,
        None => {
            help();
            return;
        }
    };
    let name = match args().nth(2) {
        Some(mut name) => {
            if name.len() > 3 {
                warn!("Name is too long, truncating to 3 characters");
                name.truncate(3);
                name
            } else {
                name
            }
        },
        None => {
            help();
            return;
        }
    };

    // Create listener
    let addr = format!("{}:{}", LISTEN_ADDR, &port);
    let listener = TcpListener::bind(&addr).await.unwrap();

    // Create routing entry for self
    let routing_entry = RoutingEntry::new(
        name.clone(),
        name.clone(),
        LISTEN_ADDR.to_string(),
        port.parse::<u16>().unwrap(),
        1, // Self = 1 hop away
    );
    routingtable.lock().unwrap().add_entry(routing_entry);

    // Start listening
    info!("Listening on: {}", &addr);
    tokio::spawn(async move {
        handle_listen(listener, routingtable).await;
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

async fn handle_listen(listener: TcpListener, routingtable: Arc<Mutex<Routingtable>>) {
    info!("Listening for new connections!");

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                info!("New client connection {:?}", addr);
                let routingtable = routingtable.clone();
                tokio::spawn(async move {
                    process(socket, routingtable).await;
                });
            }
            Err(e) => {
                warn!("Client connection failed = {:?}", e);
            }
        }
    }
}

async fn process(socket: TcpStream, routingtable: Arc<Mutex<Routingtable>>) {
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
