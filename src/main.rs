use tokio::net::{TcpListener, TcpStream};
use std::{collections::HashMap, io, env::args};
use bytes::{BytesMut, BufMut};
use std::sync::{Arc, Mutex};
use log::{info, trace, warn, error, debug};

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

    loop {
        
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