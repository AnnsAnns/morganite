
use log::{info, debug, warn};
use tokio::io::AsyncWriteExt;

use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::{collections::HashMap};


use crate::header::BaseHeader;
use crate::routing::{Routingtable, RoutingEntry};
use crate::{RoutingTableType, ConnectionsTableType};


pub struct Morganite {
    routingtable: RoutingTableType,
    own_name: String,
    own_port: String,
    own_addr: String,
}

impl Default for Morganite {
    fn default() -> Self {
        Morganite::new("ans".to_string(), "12345".to_string(), "127.0.0.1".to_string())
    }
}

impl Morganite {
    pub fn new(own_name: String, own_port: String, own_addr: String) -> Morganite {
        Morganite {
            routingtable: Arc::new(Mutex::new(Routingtable::new())),
            own_name,
            own_port,
            own_addr,
        }
    }

    pub async fn print_routingtable(&self) {
        let routingtable = self.routingtable.lock();
        info!("Routingtable: {}", routingtable.await);
    }

    /**
     * Adds the own name to the routing table
     */
    pub async fn add_self_to_routingtable(&mut self) {
        let entry = RoutingEntry::new(
            self.own_name.clone(),
            self.own_name.clone(),
            self.own_addr.clone(),
            self.own_port.clone().parse::<u16>().unwrap(),
            1,
        );

        self.routingtable_add(entry).await;
    }

    /**
     * Adds a new entry to the routing table
     */
    pub async fn routingtable_add(&mut self, entry: RoutingEntry) {
        self.routingtable.lock().await.add_entry(entry)
    }

    pub async fn send_routingtable(&mut self, destination: String) {
        let routingtable = self.routingtable.lock().await;
        let routingtable_bytes = routingtable.to_bytes(destination.clone());
        let entry = match routingtable.get_entry(destination.clone()) {
            Some(entry) => entry,
            None => {
                warn!("No entry found for {}", destination);
                return;
            }
        };
        let header = BaseHeader::new(
            0, 
            32,
            entry.destination.clone(),
            self.own_name.clone(),
            entry.hops,
        ).to_bytes();

        let mut connection = TcpStream::connect(entry.get_address()).unwrap();
        connection.write_all(&header).unwrap();
        connection.write_all(&routingtable_bytes).unwrap();
    }

    pub async fn connect_new(&mut self, destination: String, port: String, target_name: String) {
        debug!("Connecting to {} ({}) on port {}", target_name, destination, port);
        let entry = RoutingEntry::new(
            self.own_name.clone(),
            target_name.clone(),
            destination.clone(),
            port.parse::<u16>().unwrap(),
            1,
        );

        self.routingtable_add(entry).await;
        self.send_routingtable(target_name.clone()).await;
    }
}