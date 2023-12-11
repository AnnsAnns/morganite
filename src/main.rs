use bytes::{BufMut, BytesMut};
use connection_handler::ConnectionHandler;
use log::{debug, error, info, trace, warn};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, env::args, io};
use tokio::net::{TcpListener, TcpStream};

mod header;
mod routing;
mod connection_handler;
mod arg_parsing;
mod tui;

use header::BaseHeader;
use routing::{RoutingEntry, Routingtable};
use arg_parsing::{parse_name, parse_port};

const LISTEN_ADDR: &str = "127.0.0.1";

pub type ConnectionsTableType = Arc<Mutex<HashMap<String, TcpStream>>>;
pub type RoutingTableType = Arc<Mutex<Routingtable>>;

pub struct Morganite {
    connections: ConnectionsTableType,
    routingtable: RoutingTableType,
    own_name: String,
    own_port: String,
    own_addr: String,
}

impl Morganite {
    pub async fn new(own_name: String, own_port: String, own_addr: String) -> Morganite {
        Morganite {
            connections: Arc::new(Mutex::new(HashMap::new())),
            routingtable: Arc::new(Mutex::new(Routingtable::new())),
            own_name,
            own_port,
            own_addr,
        }
    }

    pub fn print_routingtable(&self) {
        let routingtable = self.routingtable.lock().unwrap();
        info!("Routingtable: {}", routingtable);
    }
}

#[tokio::main]
async fn main() {
    // Init logger
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let port = parse_port();

    let mut morganite = Arc::new(Mutex::new(Morganite::new(
        parse_name(), 
        port.to_string(), 
        LISTEN_ADDR.to_string(),
    ).await));

    let listener = TcpListener::bind(format!("{}:{}", LISTEN_ADDR, port)).await.unwrap();

    let mut connection_handler = ConnectionHandler::new(
        morganite.clone(),
        listener
    );

    let tui = tui::Tui::new(morganite.clone());
}

