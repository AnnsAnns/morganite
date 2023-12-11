use bytes::{BufMut, BytesMut};
use log::{debug, error, info, trace, warn};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, env::args, io};
use tokio::net::{TcpListener, TcpStream};

use crate::routing::{Routingtable, RoutingEntry};
use crate::{RoutingTableType, ConnectionsTableType};


pub struct Morganite {
    connections: ConnectionsTableType,
    routingtable: RoutingTableType,
    own_name: String,
    own_port: String,
    own_addr: String,
}

impl Morganite {
    pub async fn new(own_name: String, own_port: String, own_addr: String) -> Morganite {
        Morganite {
            connections: Arc::new(Mutex::new(HashMap::new())),
            routingtable: Arc::new(Mutex::new(Routingtable::new())),
            own_name,
            own_port,
            own_addr,
        }
    }

    pub fn print_routingtable(&self) {
        let routingtable = self.routingtable.lock().unwrap();
        info!("Routingtable: {}", routingtable);
    }

    /**
     * Adds the own name to the routing table
     */
    pub fn add_self_to_routingtable(&mut self) {
        let entry = RoutingEntry::new(
            self.own_name.clone(),
            self.own_name.clone(),
            self.own_addr.clone(),
            self.own_port.clone().parse::<u16>().unwrap(),
            1,
        );

        self.routingtable_add(entry);
    }

    /**
     * Adds a new entry to the routing table
     */
    pub fn routingtable_add(&mut self, entry: RoutingEntry) {
        self.routingtable.lock().unwrap().add_entry(entry)
    }
}