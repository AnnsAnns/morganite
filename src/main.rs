use colored::Colorize;
use listener::Listener;

use morganite::Morganite;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

mod arg_parsing;
mod listener;
mod morganite;
mod packets;
mod routing;
mod routing_ttl_manager;
mod tui;

use arg_parsing::{parse_name, parse_port};
use routing::Routingtable;

const LISTEN_ADDR: &str = "127.0.0.1";
const ROUTING_UPDATE_INTERVAL: u64 = 15;

pub type ConnectionsTableType = Arc<Mutex<HashMap<String, TcpStream>>>;
pub type RoutingTableType = Arc<Mutex<Routingtable>>;

#[tokio::main]
async fn main() {
    // Init logger
    simple_logger::SimpleLogger::new().with_level(log::LevelFilter::Info).env().init().unwrap();
    let port = parse_port();

    println!(
        "{}{}{}{}",
        "·͙⁺˚*•̩̩͙✩•̩̩͙*˚⁺‧͙⁺˚*•̩̩͙✩•̩̩͙*˚⁺‧͙⁺˚*•̩̩͙✩•̩̩͙*˚⁺‧͙".bright_green(),
        "Merry".bright_red().bold().italic().underline(),
        " X’mas".bright_white().bold().italic().underline(),
        "·͙⁺˚*•̩̩͙✩•̩̩͙*˚⁺‧͙⁺˚*•̩̩͙✩•̩̩͙*˚⁺‧͙⁺˚*•̩̩͙✩•̩̩͙*˚⁺‧͙".bright_green()
    );

    let morganite = Arc::new(Mutex::new(Morganite::new(
        parse_name(),
        port.to_string(),
        LISTEN_ADDR.to_string(),
    )));

    let listener = TcpListener::bind(format!("{}:{}", LISTEN_ADDR, port))
        .await
        .unwrap();

    let connection_handler = Listener::new(morganite.clone(), listener);

    let mut tui = tui::Tui::new(morganite.clone());

    // Spawn connection handler
    let _connectionthread = tokio::spawn(async move {
        connection_handler.await.listen().await;
    });

    // Spawn tui
    let _tuithread = tokio::spawn(async move {
        tui.handle_console().await;
    });

    // Add self to routing table
    morganite.lock().await.add_self_to_routingtable().await;

    // Start routing table TTL manager
    let mut routing_ttl_manager =
        routing_ttl_manager::RoutingTTLManager::new(morganite.lock().await.get_routingtable());
    let _routing_ttl_manager_thread = tokio::spawn(async move {
        routing_ttl_manager.start().await;
    });

    println!(
        "{}",
        format!(
            "{} {} {} {} {}",
            "R".bright_cyan(),
            "E".bright_magenta(),
            "A".bright_white(),
            "D".bright_magenta(),
            "Y".bright_cyan()
        )
        .bold()
        .on_black()
        .underline()
        .italic()
    );

    println!("{}", format!("{} Use {} to list commands",
        "(╭ರ_•́)".bold(),
        "help".bold().underline().bright_yellow(),
    ).yellow());

    loop {
        tokio::time::sleep(Duration::from_secs(ROUTING_UPDATE_INTERVAL)).await;
        morganite.lock().await.broadcast_routingtable().await;
    }

    // Wait for exit
    // join!(connectionthread, tuithread).0.unwrap();
}
