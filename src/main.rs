use tokio::net::{TcpListener, TcpStream};
use std::{collections::HashMap, io};
use bytes::{BytesMut, BufMut};
use std::sync::{Arc, Mutex};

const LISTEN_ADDR: &str = "127.0.0.1:12345";

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

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(LISTEN_ADDR).await.unwrap();

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                println!("New client connection {:?}", addr);
                tokio::spawn(async move {
                    process(socket).await;
                });
            }
            Err(e) => {
                println!("Client connection failed = {:?}", e);
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
                println!("Received {} bytes", n);
                println!("Received: {:?}", msg);
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