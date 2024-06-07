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
use crate::channel_events::ChannelEvent;
use crate::process::process;

use crate::shared::{Shared};

///TUI handling the users console inputs
pub async fn handle_console( state: Arc<Mutex<Shared>>) -> Result<(), Box<dyn Error>>{
    let stdin = tokio::io::stdin();
    let mut reader = FramedRead::new(stdin, LinesCodec::new());

    let addr = "127.0.0.1:4444".parse::<SocketAddr>()?;

    // Create a channel for this peer
    let (tx, mut rx) = mpsc::unbounded_channel();

    // Add an entry for this `Peer` in the shared state map.
    state.lock().await.peers.insert(addr, tx);
    loop {
        tokio::select! {
            //another async task has a message to be send to the user
            Some(event) = rx.recv() => {
                //display message
                tracing::info!("{:#?}", event);
            }
            //get the next line whenever theres a new one:
            result = reader.next() => match result {
                Some(Ok(line)) => {
                    //we got a new line on stdin, check for commands:
                    let line = line.trim();
                    //check line for commands:
                    if line.starts_with("exit") {
                        //exit the program
                    } else if line.starts_with("help") {
                        tracing::info!(
                        "Available commands:
                        exit,
                        help,
                        connect <IP> <port>,
                        contacts,
                        msg <IP> <port> <message>,"
                        );
                    } else if line.starts_with("connect") {
                        // connect to specified client
                        //get ip and port
                        let mut args = line.split_whitespace();
                        args.next();
                        let ip = match args.next() {
                            Some(ip) => ip,
                            None => {
                                tracing::error!("Missing ip");
                                continue;
                            }
                        };
                        let port = match args.next() {
                            Some(port) => port,
                            None => {
                                tracing::error!("Missing port");
                                continue;
                            }
                        };
                        //parse to SocketAddr
                        let destination = format!("{}:{}",ip,port);
                        let addr = match destination.parse::<SocketAddr>() {
                            Ok(socket) => socket,
                            Err(e) => {
                                tracing::error!("Error parsing destination: {}",e);
                                continue;
                            }
                        };
                        //create tcp stream
                        let stream = match TcpStream::connect(addr).await {
                            Ok(stream) => stream,
                        Err(e) => {
                            tracing::error!("Failed to connect to {}: {}", destination, e);
                            continue;
                            }   
                        };
                        // Clone a handle to the `Shared` state for the new connection.
                        let state = Arc::clone(&state);
                        //spawn asynchronous handler
                        tokio::spawn(async move {
                        tracing::info!("connected to: {}",addr);
                        if let Err(e) = process(state, stream, addr).await {
                            tracing::info!("an error occurred; error = {:?}", e);
                        }
                    });
                
                    } else if line.starts_with("contacts") {
                        // diplay the routing table
                        {
                            let mut lock = state.lock().await;
                            for peer in lock.peers.iter_mut() {
                                if peer.0 != &addr {
                                    tracing::info!("connected to: {}",peer.0);
                                }
                            }
                        }
                    }else if line.starts_with("msg") {
                        // msg client, if known
                        let mut args = line.split_whitespace();
                        //skip 'msg':
                        args.next(); 
                        //get destination:
                        let ip = match args.next() {
                            Some(ip) => ip,
                            None => {
                                tracing::error!("Missing ip");
                                continue;
                            }
                        };
                        let port = match args.next() {
                            Some(port) => port,
                            None => {
                                tracing::error!("Missing port");
                                continue;
                            }
                        };
                        //get message:
                        let  message = args.collect::<Vec<&str>>().join(" ");
                        //parse to SocketAddr
                        let destination = format!("{ip}:{port}");
                        let addr = match destination.parse::<SocketAddr>() {
                            Ok(socket) => socket,
                            Err(e) => {
                                tracing::error!("Error parsing destination: {}",e);
                                continue;
                            }
                        };
                        //send message:
                        //get destination from list of peers
                        {
                            let lock = state.lock().await;
                            let peer = match lock.peers.get(&addr) {
                                Some(peer) => peer,
                                None => { 
                                    tracing::error!("Unknown destination: {}",addr);
                                    continue;
                                }
                            };
                            if let Err(e) = peer.send(ChannelEvent::Message(message)) {
                                tracing::info!("Error sending your message. error = {:?}", e);
                            }
                        }
                    } else {
                        tracing::error!("Unknown command: {}", line);
                    }
                }
                // An error occurred.
                Some(Err(e)) => {
                    tracing::error!( "an error occurred while processing stdin. error = {:?}",e);
                }
                // The stream has been exhausted.
                None => break Ok(()),
            }
        }
    }
}