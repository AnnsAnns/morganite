use tokio::net::TcpStream;
use tokio::sync::{mpsc, Mutex};

use crate::channel_events::{ChannelEvent, Commands};
use crate::process::process;

use std::error::Error;

use std::net::SocketAddr;
use std::sync::Arc;

use crate::shared::{RoutingTableEntry, Shared};

///TUI handling the users console inputs
pub async fn handle_console(state: Arc<Mutex<Shared>>) -> Result<(), Box<dyn Error>> {
    let addr = "127.0.0.1:4444".parse::<SocketAddr>()?;
    let client_addr = state
        .lock()
        .await
        .listener_addr
        .clone()
        .parse::<SocketAddr>()
        .unwrap();

    // Create a channel for this peer
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Add an entry for this `Peer` in the shared state map.
    state.lock().await.peers.insert(addr, tx);

    // Create a channel for the console input
    let (command_sender, command_receiver) = std::sync::mpsc::channel::<ChannelEvent>();

    // Sleep for 5 seconds to allow the server to start up
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    // Send the command receiver to the TUI
    match state
        .lock()
        .await
        .console_input_sender
        .send(ChannelEvent::CommandReceiver(command_sender))
    {
        Ok(_) => {
            tracing::info!("Sent command receiver to TUI");
        }
        Err(e) => {
            tracing::error!("Error sending command receiver to TUI: {:?}", e);
            // Exit the application
            std::process::exit(1);
        }
    }

    loop {
        // We're locking the cmd receiver, possibly for a long time but this is actually fine
        // Since this is the only place where we're reading from the cmd receiver
        // Still required a Mutex<> since the compiler doesn't know that

        //tracing::debug!("Waiting for console input");

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
                                    {
                                        let mut lock = state.lock().await;
                                        for entry in lock.routing_table.values_mut() {
                                            entry.hop_count = 32;
                                        }
                                        for entry in lock.peers.values_mut() {
                                            
                                            if let Err(e) = entry.send(ChannelEvent::Routing(6)) {
                                                tracing::error!("Error sending STU to quit gracefully: {:?}", e);
                                            }
                                        }

                                    }
                                    std::process::exit(0);
                                },
                                Commands::Contacts => {
                                    // Display the routing table
                                    //tracing::debug!("Displaying routing table");
                                        let state_lock = state.lock().await;
                                        let routing_table = state_lock.routing_table.clone();

                                        //tracing::debug!("Sent routing table to TUI");
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
                                    //check wether there is already a direct connection to the target client:
                                    let already_connected: bool;
                                    {
                                        let lock = state.lock().await;
                                        match lock.routing_table.get(&addr) {
                                            Some(direct) => {
                                                already_connected = direct.hop_count == 1;
                                            },
                                            None => {
                                                already_connected = false;
                                            },
                                        };
                                    }
                                    if !already_connected {
                                        // Connect to specified client
                                        tracing::debug!("Connecting to: {}", addr);

                                        // Create TCP stream
                                        let stream = match TcpStream::connect(addr).await {
                                            Ok(stream) => stream,
                                            Err(e) => {
                                                tracing::error!("Failed to connect to {}: {}", addr, e);
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
                                    }
                                },
                                Commands::Broadcast(message) => {
                                    // Broadcast message to all clients
                                    tracing::debug!("Broadcasting message: {}", message);

                                    // Get the list of all entries in the routing table
                                    let routing_table = {
                                        let lock = state.lock().await;
                                        lock.routing_table.clone()
                                    };

                                    // Send the message to all clients via next hop
                                    for (addr, routing_table_entry) in routing_table.iter() {
                                        // Get the channel to the next client/destination on the route
                                        let target = if routing_table_entry.next == client_addr {
                                            addr.clone()
                                        } else {
                                            routing_table_entry.next
                                        };

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
                                        if let Err(e) = peer.send(ChannelEvent::Message(message.clone(), addr.clone())) {
                                            tracing::info!("Error sending your message. error = {:?}", e);
                                        }
                                    }
                                },

                                Commands::Message(addr, message) => {
                                    // Send message to specified client
                                    tracing::debug!("Sending message to: {}", addr);

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
                                    let target = if routing_table_entry.next == client_addr {
                                         addr
                                    } else {
                                         routing_table_entry.next
                                    };

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

        //tracing::debug!("Finished processing command");
    }
}
