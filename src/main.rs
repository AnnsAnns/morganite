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

mod protocol;
mod swag_coding;
mod channel_events;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // construct a subscriber that prints formatted traces to stdout
    let subscriber = tracing_subscriber::FmtSubscriber::new();
    // use that subscriber to process traces emitted after this point
    tracing::subscriber::set_global_default(subscriber)?;

    // Create the shared state. This is how all the peers communicate.
    //
    // The server task will hold a handle to this. For every new client, the
    // `state` handle is cloned and passed into the task that processes the
    // client connection.
    let state = Arc::new(Mutex::new(Shared::new()));

    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6142".to_string());

    // Bind a TCP listener to the socket address.
    //
    // Note that this is the Tokio TcpListener, which is fully async.
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("server running on {}", addr);

    //TUI Task
    //create a Task that runs the TUI code with access to Shared to allow the creation of new connections
    let state_tui = Arc::clone(&state);
    tokio::spawn(async move {
        tracing::debug!("created TUI task");
        if let Err(e) = handle_console(state_tui).await {
            tracing::info!("an error occurred; error = {:?}", e);
        }
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
            if let Err(e) = process(state, stream, addr).await {
                tracing::info!("an error occurred; error = {:?}", e);
            }
        });
    }
}

/// Shorthand for the transmit half of the message channel.
type Tx = mpsc::UnboundedSender<String>;

/// Shorthand for the receive half of the message channel.
type Rx = mpsc::UnboundedReceiver<String>;

/// Data that is shared between all peers in the chat server.
///
/// This is the set of `Tx` handles for all connected clients. Whenever a
/// message is received from a client, it is broadcasted to all peers by
/// iterating over the `peers` entries and sending a copy of the message on each
/// `Tx`.
struct Shared {
    peers: HashMap<SocketAddr, Tx>,
}

/// The state for each connected client.
struct Peer {
    /// The TCP socket wrapped with the `SwagCoder` codec, defined below.
    ///
    /// This handles sending and receiving data on the socket. When using
    /// `Lines`, we can work at the line level instead of having to manage the
    /// raw byte operations.
    swag_coder: Framed<TcpStream, SwagCoder>,

    /// Receive half of the message channel.
    ///
    /// This is used to receive messages from peers. When a message is received
    /// off of this `Rx`, it will be written to the socket.
    rx: Rx,
}

impl Shared {
    /// Create a new, empty, instance of `Shared`.
    fn new() -> Self {
        Shared {
            peers: HashMap::new(),
        }
    }

    /// Send a `LineCodec` encoded message to every peer, except
    /// for the sender.
    async fn broadcast(&mut self, sender: SocketAddr, message: &str) {
        for peer in self.peers.iter_mut() {
            if *peer.0 != sender {
                let _ = peer.1.send(message.into());
            }
        }
    }
}

impl Peer {
    /// Create a new instance of `Peer`.
    async fn new(
        state: Arc<Mutex<Shared>>,
        swag_coder: Framed<TcpStream, SwagCoder>,
    ) -> io::Result<Peer> {
        // Get the client socket address
        let addr = swag_coder.get_ref().peer_addr()?;

        // Create a channel for this peer
        let (tx, rx) = mpsc::unbounded_channel();

        // Add an entry for this `Peer` in the shared state map.
        state.lock().await.peers.insert(addr, tx);
        tracing::info!("added address: {}",addr);
        Ok(Peer { swag_coder, rx })
    }
}
///TUI handling the users console inputs
async fn handle_console( state: Arc<Mutex<Shared>>) -> Result<(), Box<dyn Error>>{
    // TODO
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
            Some(msg) = rx.recv() => {
                //display message
                tracing::info!("{}", msg);
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
                                tracing::info!("connected to: {}",peer.0);
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
                            if let Err(e) = peer.send(message.into()) {
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

/// Process an individual chat client
async fn process(
    state: Arc<Mutex<Shared>>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let swag_coder = Framed::new(stream, SwagCoder::new());

    // Register our peer with state which internally sets up some channels.
    let mut peer = Peer::new(state.clone(), swag_coder).await?;

    // A client has connected, let's let everyone know.
    {
        let mut state = state.lock().await;
        let msg = format!("{addr} has joined the chat");
        state.broadcast(addr, &msg).await;
    }

    // Process incoming messages until our stream is exhausted by a disconnect.
    loop {
        tokio::select! {
            // A message was received from a peer. Send it to the current user.
            Some(msg) = peer.rx.recv() => {

                tracing::info!("received message from peer: {:#?}", msg);
                // let msg = msg?;
                // peer.swag_coder.send(msg).await?;
            }
            result = peer.swag_coder.next() => match result {
                // A message was received from the current user, we should
                // broadcast this message to the other users.
                Some(Ok(packet)) => {
                    let mut state = state.lock().await;
                    let msg = format!("{}: {:#?}", addr, packet);
                    //handle message from others
                    state.broadcast(addr, &msg).await;
                }
                // An error occurred.
                Some(Err(e)) => {
                    tracing::error!(
                        "an error occurred while processing messages for {}; error = {:?}",
                        addr,
                        e
                    );
                }
                // The stream has been exhausted.
                None => break,
            },
        }
    }

    // If this section is reached it means that the client was disconnected!
    // Let's let everyone still connected know about it.
    {
        let mut state = state.lock().await;
        state.peers.remove(&addr);

        let msg = format!("{} has left the chat", addr);
        tracing::info!("{}", msg);
        state.broadcast(addr, &msg).await;
    }

    Ok(())
}