
use connection_handler::ConnectionHandler;

use morganite::Morganite;

use std::sync::Arc;
use tokio::sync::Mutex;
use std::{collections::HashMap};
use tokio::net::{TcpListener, TcpStream};
use tokio::join;

mod header;
mod routing;
mod connection_handler;
mod arg_parsing;
mod tui;
mod morganite;


use routing::{Routingtable};
use arg_parsing::{parse_name, parse_port};

const LISTEN_ADDR: &str = "127.0.0.1";

pub type ConnectionsTableType = Arc<Mutex<HashMap<String, TcpStream>>>;
pub type RoutingTableType = Arc<Mutex<Routingtable>>;

#[tokio::main]
async fn main() {
    // Init logger
    simple_logger::SimpleLogger::new().env().init().unwrap();
    let port = parse_port();

    let morganite = Arc::new(Mutex::new(Morganite::new(
        parse_name(), 
        port.to_string(), 
        LISTEN_ADDR.to_string(),
    )));

    let listener = TcpListener::bind(format!("{}:{}", LISTEN_ADDR, port)).await.unwrap();

    let connection_handler = ConnectionHandler::new(
        morganite.clone(),
        listener
    );

    let tui = tui::Tui::new(morganite.clone());

    // Spawn connection handler
    let connectionthread = tokio::spawn(async move {
        connection_handler.await.listen().await;
    });
    
    // Spawn tui
    let tuithread = tokio::spawn(async move {
        tui.handle_console().await;
    });

    // Add self to routing table
    morganite.lock().await.add_self_to_routingtable().await;

    // Print routing table
    morganite.lock().await.print_routingtable().await;

    // Wait for exit
    join!(connectionthread, tuithread).0.unwrap();
}

