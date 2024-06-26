use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt;
use tokio_util::codec::{Framed, FramedRead, LinesCodec};

use crate::channel_events::{ChannelEvent, Commands};
use crate::process::process;
use futures::SinkExt;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;

use crate::protocol::CR;
use crate::shared::{RoutingTableEntry, Shared};

///TUI handling the users console inputs
pub async fn handle_console(state: Arc<Mutex<Shared>>) -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:4444".parse::<SocketAddr>()?;
    let client_addr = state.lock().await.listener_addr.clone().parse::<SocketAddr>().unwrap();

    // Create a channel for this peer
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Add an entry for this `Peer` in the shared state map.
    state.lock().await.peers.insert(addr, tx);

    // Create a channel for the console input
    let (command_sender, mut command_receiver) = std::sync::mpsc::channel::<ChannelEvent>();

    // Sleep for 5 seconds to allow the server to start up
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    // Send the command receiver to the TUI
    state
        .lock()
        .await
        .console_input_sender
        .send(ChannelEvent::CommandReceiver(command_sender))
        .unwrap();

    loop {
        // We're locking the cmd receiver, possibly for a long time but this is actually fine
        // Since this is the only place where we're reading from the cmd receiver
        // Still required a Mutex<> since the compiler doesn't know that

        tracing::debug!("Waiting for console input");

        // Create timer for tokio::select to prevent blocking
        let timer = tokio::time::sleep(tokio::time::Duration::from_millis(100));

        tokio::select! {
            // Check for new peer events
            Some(event) = rx.recv() => {
                tracing::debug!("Received event: {:?}", event);

                // Send the event to TUI
                if let Err(e) = state.lock().await.console_input_sender.send(event) {
                    tracing::error!("Error sending event to TUI: {:?}", e);
                }
            },
            // Check for new console commands
            _ = timer => {
                if let Ok(event) = command_receiver.try_recv() {
                    // If the channel event is type routing, ignore it
                    tracing::debug!("Received event: {:?}", event);

                    match event {
                        ChannelEvent::MessageToTUI(message, name, addr) => {
                            // Send message to TUI
                            tracing::debug!("Sending message to TUI");

                            // Send the message to the channel
                            if let Err(e) = state.lock().await.console_input_sender.send(ChannelEvent::MessageToTUI(message, name, addr)) {
                                tracing::info!("Error sending your message. error = {:?}", e);
                            }
                        },
                        ChannelEvent::Command(cmd) => {
                            tracing::debug!("Received command: {:?}", cmd);

                            match cmd {
                                Commands::Quit => {
                                    // Quit the application
                                    tracing::debug!("Quitting application");
                                    // Quit all tokio tasks
                                    // @TODO: Gracefully announce to all peers that we are quitting?
                                    std::process::exit(0);
                                },
                                Commands::Contacts => {
                                    // Display the routing table
                                    tracing::debug!("Displaying routing table");
                                        let mut state_lock = state.lock().await;
                                        let routing_table = state_lock.routing_table.clone();

                                        tracing::debug!("Sent routing table to TUI");
                                        if let Err(e) = state_lock.console_input_sender.send(ChannelEvent::Contacts(routing_table)) {
                                            tracing::error!("Error sending routing table to TUI: {:?}", e);
                                        }
                                },
                                Commands::SetOwnNick(nickname) => {
                                    // Set the nickname
                                    tracing::debug!("Setting nickname to: {}", nickname);
                                    let mut state_lock = state.lock().await;
                                    state_lock.nickname = nickname;
                                },
                                Commands::Connect(addr) => {
                                    // Connect to specified client
                                    tracing::debug!("Connecting to: {}", addr);
                                    state.lock().await.console_input_sender.send(ChannelEvent::LogToTerminal(format!("Connecting to: {}", addr))).unwrap();

                                    // Create TCP stream
                                    let stream = match TcpStream::connect(addr).await {
                                        Ok(stream) => stream,
                                        Err(e) => {
                                            tracing::error!("Failed to connect to {}: {}", addr, e);
                                            state.lock().await.console_input_sender.send(ChannelEvent::LogToTerminal(format!("Failed to connect to {}: {}", addr, e))).unwrap();
                                            continue;
                                        }
                                    };

                                    // Clone a handle to the `Shared` state for the new connection.
                                    let proccess_state = Arc::clone(&state);

                                    //add new connection to routing table
                                    {
                                    let mut lock = state.lock().await;
                                    lock.routing_table.insert(addr, RoutingTableEntry {next:addr, hop_count: 1, ttl: true});
                                    }
                                    // Spawn asynchronous handler
                                    tokio::spawn(async move {
                                        tracing::info!("Connected to: {}", addr);
                                        if let Err(e) = process(proccess_state, stream, addr, true).await {
                                            tracing::info!("An error occurred; error = {:?}", e);
                                        }
                                    });
                                },
                                Commands::Message(addr, message) => {
                                    // Send message to specified client
                                    tracing::debug!("Sending message to: {}", addr);

                                    // Get the client socket address
                                    let client_addr = client_addr;

                                    // Get the routing table entry for the destination
                                    let routing_table_entry = {
                                        let lock = state.lock().await;
                                        match lock.routing_table.get(&addr) {
                                            Some(routing_table_entry) => routing_table_entry.clone(),
                                            None => {
                                                tracing::error!("No route to destination: {} available", addr);
                                                continue;
                                            }
                                        }
                                    };

                                    // Get the channel to the next client/destination on the route
                                    let target;
                                    if routing_table_entry.next == client_addr {
                                        target = addr;
                                    } else {
                                        target = routing_table_entry.next;
                                    }

                                    let peer = {
                                        let lock = state.lock().await;
                                        match lock.peers.get(&target) {
                                            Some(peer) => peer.clone(),
                                            None => {
                                                tracing::error!("No channel to destination: {} available", target);
                                                continue;
                                            }
                                        }
                                    };

                                    // Send the message to the channel
                                    if let Err(e) = peer.send(ChannelEvent::Message(message, addr)) {
                                        tracing::info!("Error sending your message. error = {:?}", e);
                                    }
                                },
                                _ => {
                                    tracing::error!("Unknown command: {:#?}", cmd);
                                }
                            }
                        },
                        _ => {
                            tracing::error!("Received non-command event in console middleware");
                        },
                    }
                }
            },

        }

        tracing::debug!("Finished processing command");
    }
}