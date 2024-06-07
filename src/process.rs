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

use crate::peer::Peer;
use crate::protocol::Packet;
use crate::protocol::shared_header::SharedHeader;
use crate::protocol::routed_packet::RoutedPacket;
use crate::shared::Shared;
use crate::{channel_events, swag_coding};

/// Process an individual chat client
pub async fn process(
    state: Arc<Mutex<Shared>>,
    stream: TcpStream,
    addr: SocketAddr,
) -> Result<(), Box<dyn Error>> {
    let local_addr = match stream.local_addr() {
        Ok(addr) => addr,
        Err(e) => { 
            tracing::info!("an error occurred; error = {:?}", e);
            panic!(); //TODO maybe different error handling
        },
    };
    let swag_coder = Framed::new(stream, SwagCoder::new());
    
    // Register our peer with state which internally sets up some channels.
    let mut peer = Peer::new(state.clone(), swag_coder).await?;
    // A client has connected, let's let everyone know.
    {
        let mut state = state.lock().await;
        let msg = tracing::info!("{addr} has joined the chat");
        state.broadcast(addr, &ChannelEvent::Join(addr.to_string())).await;
    }

    // Process incoming messages until our stream is exhausted by a disconnect.
    loop {
        tokio::select! {
            Some(event) = peer.rx.recv() => {
                tracing::info!("Received Event: {:#?}", event);
                let header = SharedHeader {
                    source_ip: local_addr.ip().to_string(),
                    source_port: local_addr.port().to_string(),
                    destination_ip: addr.ip().to_string(),
                    destination_port: addr.port().to_string(),
                    ttl: 16,
                };
                // create packet
                match event {
                    ChannelEvent::Message(msg) => {
                        let routed_packet = RoutedPacket {
                            header,
                            nickname: "TODO".to_string(),
                            data: msg,
                        };
                        peer.swag_coder.send(Packet::RoutedPacket(routed_packet)).await?;
                    },
                    ChannelEvent::Forward(packet) => {
                        peer.swag_coder.send(packet).await?;
                    }
                    _ => tracing::error!("Received Event: {:#?} is not implemented!", event),
                } 
                


            }
            result = peer.swag_coder.next() => match result {
                // A message was received from the current user, we should
                // broadcast this message to the other users.
                Some(Ok(packet)) => {
                    let mut state = state.lock().await;
                    tracing::info!("{}: {:#?}", addr, packet);
                    //handle message from others
                    state.broadcast(addr, &ChannelEvent::Unknown).await;
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
        state.broadcast(addr, &ChannelEvent::Leave(addr.to_string())).await;
    }

    Ok(())
}