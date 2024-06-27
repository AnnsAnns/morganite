use console_middleware::handle_console;
use shared::Shared;

use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};

use tracing::Level;

use std::error::Error;
use std::{env, thread};

use crate::process::process;
use std::sync::Arc;

// TUI

mod channel_events;
mod console_middleware;
mod heartbeat;
mod peer;
mod process;
mod protocol;
mod shared;
mod swag_coding;
mod tui;

/// Use Tokio Runtime, Multi-Threaded with a Thread Pool based on the number of cores available
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Check whether ./logs/morganite.log exists & delete it if it does
    if std::fs::metadata("logs/morganite.log").is_ok() {
        std::fs::remove_file("logs/morganite.log")?;
    }

    // construct a subscriber that logs formatted traces to file
    let file_appender = tracing_appender::rolling::never("logs", "morganite.log");
    // use that subscriber to process traces emitted after this point
    tracing_subscriber::fmt()
        .compact()
        .with_writer(file_appender)
        .with_max_level(Level::DEBUG)
        .init();

    // Create the shared state. This is how all the peers communicate.
    //
    // The server task will hold a handle to this. For every new client, the
    // `state` handle is cloned and passed into the task that processes the
    // client connection.
    let (console_input_tx, console_input_rx) = mpsc::unbounded_channel();
    let state = Arc::new(Mutex::new(Shared::new(console_input_tx)));

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6142".to_string());

    // Bind a TCP listener to the socket address.
    //
    // Note that this is the Tokio TcpListener, which is fully async.
    let listener = TcpListener::bind(&addr).await?;
    //add listener addr to shared space
    state.lock().await.listener_addr = addr.clone();
    tracing::info!("server running on {}", addr);

    // Spawn heartbeat task
    let heartbeat_state = Arc::clone(&state);
    tokio::spawn(async move {
        tracing::debug!("created heartbeat task");
        if let Err(e) = heartbeat::heartbeat(heartbeat_state).await {
            tracing::info!("an error occurred; error = {:?}", e);
        }
    });

    // Spawn console middleware
    let console_state = Arc::clone(&state);
    tokio::spawn(async move {
        tracing::debug!("created console middleware task");
        if let Err(e) = handle_console(console_state).await {
            tracing::info!("an error occurred; error = {:?}", e);
        }
    });

    // Spawn new thread for TUI
    thread::spawn(move || tui::tui(console_input_rx));

    //Loop accepting new connections from other clients creating a task for each of them handling their messages
    loop {
        // Asynchronously wait for an inbound TcpStream.
        let (stream, addr) = listener.accept().await?;

        // Clone a handle to the `Shared` state for the new connection.
        let state = Arc::clone(&state);

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            tracing::info!("accepted connection to {}", addr);
            if let Err(e) = process(state, stream, addr, false).await {
                tracing::info!("an error occurred; error = {:?}", e);
            }
        });
    }
}
