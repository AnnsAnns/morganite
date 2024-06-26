use channel_events::ChannelEvent;
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

use crate::shared::{RoutingTableEntry, Rx, Shared};
use crate::{channel_events, swag_coding};

/// The state for each connected client.
pub struct Peer {
    /// The TCP socket wrapped with the `SwagCoder` codec, defined below.
    ///
    /// This handles sending and receiving data on the socket. When using
    /// `SwagCoder`, we can work at the packet level instead of having to manage the
    /// raw byte operations.
    pub swag_coder: Framed<TcpStream, SwagCoder>,

    /// Receive half of the message channel.
    ///
    /// This is used to receive messages from peers. When a message is received
    /// off of this `Rx`, it will be written to the socket.
    pub rx: Rx,
}

impl Peer {
    /// Create a new instance of `Peer`.
    pub async fn new(
        state: Arc<Mutex<Shared>>,
        swag_coder: Framed<TcpStream, SwagCoder>,
    ) -> io::Result<Peer> {
        // Get the client socket address
        let addr = swag_coder.get_ref().peer_addr()?;
        // Create a channel for this peer
        let (tx, rx) = mpsc::unbounded_channel();

        // Add an entry for this `Peer` in the shared state map and the routing table.
        {
            let mut lock = state.lock().await;
            lock.peers.insert(addr, tx);
            lock.routing_table.insert(addr, RoutingTableEntry {next:addr, hop_count: 1, ttl: true});
        }
       
        tracing::info!("added address: {}",addr);
        Ok(Peer { swag_coder, rx })
    }
}