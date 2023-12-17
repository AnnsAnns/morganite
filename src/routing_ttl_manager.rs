use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{morganite::Morganite, RoutingTableType};



pub struct RoutingTTLManager {
    routing_table: RoutingTableType,
}

impl RoutingTTLManager {
    pub fn new(routing_table: RoutingTableType) -> RoutingTTLManager {
        RoutingTTLManager {
            routing_table,
        }
    }

    /**
     * Decrements the TTL of all routing entries in the routing table by 1.
     */
    pub async fn start(&mut self) {
        loop {
            self.routing_table.lock().await.remove_expired_entries().await;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            self.routing_table.lock().await.decrement_ttl().await;
        }
    }
}