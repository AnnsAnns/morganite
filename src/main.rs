use channel_events::ChannelEvent;
use console_middleware::handle_console;
use shared::Shared;
use swag_coding::SwagCoder;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, FramedRead, LinesCodec};

use futures::SinkExt;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use crate::process::process;

// TUI
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::{CrosstermBackend, Stylize, Terminal},
    widgets::Paragraph,
};

mod protocol;
mod swag_coding;
mod channel_events;
mod console_middleware;
mod heartbeat;
mod process;
mod peer;
mod shared;
mod tui;

/// Use Tokio Runtime, Multi-Threaded with a Thread Pool based on the number of cores available
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // construct a subscriber that logs formatted traces to file
    let file_appender = tracing_appender::rolling::never("logs", "morganite.log");
    // use that subscriber to process traces emitted after this point
    tracing_subscriber::fmt()
        .fmt(f)
        .with_writer(file_appender)
        .init();

    // Create the shared state. This is how all the peers communicate.
    //
    // The server task will hold a handle to this. For every new client, the
    // `state` handle is cloned and passed into the task that processes the
    // client connection.
    let (console_input_tx, console_input_rx) = mpsc::unbounded_channel();
    let (console_cmd_tx, console_cmd_rx) = mpsc::unbounded_channel();
    let state = Arc::new(Mutex::new(Shared::new(console_input_tx, console_cmd_rx)));

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6142".to_string());

    // Bind a TCP listener to the socket address.
    //
    // Note that this is the Tokio TcpListener, which is fully async.
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("server running on {}", addr);

    // Spawn heartbeat task
    let heartbeat_state = Arc::clone(&state);
    tokio::spawn(async move {
        tracing::debug!("created heartbeat task");
        if let Err(e) = heartbeat::heartbeat(heartbeat_state).await {
            tracing::info!("an error occurred; error = {:?}", e);
        }
    });

    let new_tui = Arc::clone(&state);
    tokio::spawn(async move {
        tracing::debug!("created TUI task");
        if let Err(e) = tui::tui(console_input_rx, console_cmd_tx) {
            tracing::info!("an error occurred; error = {:?}", e);
        }
        // If the TUI task ends, the program should end
        std::process::exit(0);
    });

    //Loop accepting new connections from other clients creating a task for each of them handling their messages
    loop {
        // Asynchronously wait for an inbound TcpStream.
        let (stream, addr) = listener.accept().await?;

        // Clone a handle to the `Shared` state for the new connection.
        let state = Arc::clone(&state);

        // Spawn our handler to be run asynchronously.
        tokio::spawn(async move {
            tracing::info!("accepted connection to {}",addr);
            if let Err(e) = process(state, stream, addr, false).await {
                tracing::info!("an error occurred; error = {:?}", e);
            }
        });
    }
}