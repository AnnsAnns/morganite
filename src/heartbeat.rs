use std::{sync::Arc, time::Duration};

use tokio::sync::{mpsc::error::SendError, Mutex};

use crate::{channel_events::ChannelEvent, protocol::{SCC, STU}, shared::Shared};

pub const POISE_UNREACHABLE: i32 = 32;

pub async fn heartbeat(state: Arc<Mutex<Shared>>) -> Result<(), SendError<ChannelEvent>> {
    loop {
        // Send SCC to all peers
        {
            let lock = state.lock().await;
            for (_, tx) in &lock.peers {
                tx.send(ChannelEvent::Routing(SCC))?;
            }
        }

        // Give them 1 second to respond
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Check for ttl flag
        {
            let mut lock = state.lock().await;
            for (addr, entry) in &mut lock.routing_table {
                if !entry.ttl {
                    entry.hop_count = POISE_UNREACHABLE;
                }

                entry.ttl = false;
            }
        }

        // Send STU to all peers
        {
            let lock = state.lock().await;
            for (_, tx) in &lock.peers {
                tx.send(ChannelEvent::Routing(STU))?;
            }
        }

        // Sleep 10 seconds
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}