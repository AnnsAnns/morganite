use bytes::{BufMut, BytesMut};
use log::{debug, info, warn};


use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    packets::{
        header::{BaseHeader, PacketType, BASE_HEADER_SIZE},
        routing_entry::RoutingEntry,
        Packet, connection::ConnectionPacket,
    },
    routing::Routingtable,
    RoutingTableType,
};

pub struct Morganite {
    routingtable: RoutingTableType,
    own_name: String,
    own_port: String,
    own_addr: String,
}

impl Default for Morganite {
    fn default() -> Self {
        Morganite::new(
            "ans".to_string(),
            "12345".to_string(),
            "127.0.0.1".to_string(),
        )
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

    /**
     * Prints the routing table to the console
     */
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
        debug!(
            "New Entry \"{}\" from \"{}\" ({}:{})",
            &entry.destination, &entry.info_source, &entry.ip, &entry.port);
        self.routingtable.lock().await.add_entry(entry).await;
    }

    /**
     * Broadcasts the routing table to all direct sources
     */
    pub async fn broadcast_routingtable(&self) {
        let own_name = self.own_name.clone();
        //debug!("Broadcasting routingtable");
        let direct_sources = self.routingtable.lock().await
            .get_direct_sources(own_name)
            .await;

        for entry in direct_sources {
            self.send_routingtable(entry).await;
        }
    }

    /**
     * Sends the routing table to the given destination
     * @param destination The destination to send the routing table to (name)
     */
    pub async fn send_routingtable(&self, destination: String) {
        if destination == self.own_name {
            return;
        }

        // Lock routing table to get entry
        let routingtable = self.routingtable.lock().await;

        let entry = match routingtable.get_entry(destination.clone()) {
            Some(entry) => entry,
            None => {
                warn!("No entry found for {}", destination);
                return;
            }
        };
        let header = BaseHeader::new(
            PacketType::Routing,
            32,
            entry.destination.clone(),
            self.own_name.clone(),
            entry.hops,
        )
        .to_bytes();

        let routingtable_bytes = routingtable.to_bytes(
            "".to_string(),
            self.own_addr.clone(),
            self.own_port.clone().parse::<u16>().unwrap(),
            self.own_name.clone(),
        );

        debug!("Routingtable bytes: {:#?}", routingtable_bytes.clone());

        debug!("Sending routingtable to {}", destination);
        let mut msg = BytesMut::with_capacity(1024);
        msg.put(header.clone());
        msg.put(routingtable_bytes.clone());

        let packet = Packet::create_crc32(msg).await;

        if !packet.verify_self().await {
            warn!("Invalid checksum was generated (Something is seriously wrong lol)");
            return;
        }

        let mut connection = match TcpStream::connect(entry.get_address()) {
            Ok(connection) => connection,
            Err(e) => {
                warn!("Could not connect to {}: {} - Removing from Routing Table", entry.get_address(), e);
                self.routingtable.lock().await.remove_entry(destination.clone()).await;
                return;
            }
        };
        connection.write_all(&packet.to_bytes()).unwrap();
        connection.flush().unwrap();
        debug!("Sent routingtable to {}", destination);
    }

    /**
     * Removes the entry with the given destination
     * @param destination The destination to remove
     */
    pub async fn remove_entry(&mut self, destination: String) {
        self.routingtable.lock().await.remove_entry(destination).await;
    }

    /**
     * Updates the routing table with the given bytes
     * @param bytes The bytes to update the routing table with
     * @param ip The ip of the sender
     */
    pub async fn update_routing_table(&mut self, bytes: BytesMut, ip: String) {
        debug!("Updating routing table with {}", ip);
        let header = match BaseHeader::from_bytes(bytes.clone()) {
            Some(header) => header,
            None => {
                warn!("Invalid header");
                return;
            }
        };
        self.routingtable.lock().await.clear_from(header.get_source()); // clear all entries from the source
        let routingtable_bytes = bytes[BASE_HEADER_SIZE..].to_vec();
        let total_entries = routingtable_bytes[0];
        let mut offset = 1;
        for _ in 0..total_entries {
            let entry_bytes = routingtable_bytes[offset..offset + 9].to_vec();
            let entry =
                RoutingEntry::from_bytes(BytesMut::from(entry_bytes.as_slice()), ip.clone());
            debug!("Adding entry: {:#?}", entry);
            if entry.info_source == self.own_name
                || entry.destination == self.own_name
                || entry.destination == entry.info_source
            {
                continue;
            }
            self.routingtable_add(entry).await;
            offset += 9;
        }
    }

    /**
     * Connects to the given destination
     * @param destination The destination to connect to (IP)
     * @param port The port to connect to
     * @param target_name The name of the target
     */
    pub async fn connect_new(&mut self, destination: String, port: String, target_name: String) {
        debug!(
            "Connecting to {} ({}) on port {}",
            target_name, destination, port
        );
        let entry = RoutingEntry::new(
            self.own_name.clone(),
            target_name.clone(),
            destination.clone(),
            port.parse::<u16>().unwrap(),
            1,
        );

        self.routingtable_add(entry).await;
        self.send_connectionpacket(target_name.clone(), true).await;
        self.send_routingtable(target_name.clone()).await;
    }

    pub async fn send_connectionpacket(&mut self, target: String, first: bool) {
        let routingtable = self.routingtable.lock().await;
        let entry = match routingtable.get_entry(target.clone()) {
            Some(entry) => entry,
            None => {
                warn!("No entry found for {}", target);
                return;
            }
        };
        let mut connection = TcpStream::connect(entry.get_address()).unwrap();
        let header = BaseHeader::new(
            PacketType::Connection,
            32,
            entry.destination.clone(),
            self.own_name.clone(),
            entry.hops,
        );
        let connection_packet = 
            ConnectionPacket::new(self.own_name.clone(), self.own_port.clone().parse::<u16>().unwrap(), first).to_bytes()
        ;
        let mut msg = BytesMut::with_capacity(1024);
        msg.put(header.to_bytes());
        msg.put(connection_packet.clone());
        let packet = Packet::create_crc32(msg).await;
        connection.write_all(&packet.to_bytes()).unwrap();
        connection.flush().unwrap();
        debug!("Sent connection packet to {}", target);
    }

    pub async fn get_port_of(&self, name: String) -> Option<u16> {
        let routingtable = self.routingtable.lock().await;
        let entry = match routingtable.get_entry(name.clone()) {
            Some(entry) => entry,
            None => {
                warn!("No entry found for {}", name);
                return None;
            }
        };
        Some(entry.port)
    }

    pub async fn get_ip_of(&self, name: String) -> Option<String> {
        let routingtable = self.routingtable.lock().await;
        let entry = match routingtable.get_entry(name.clone()) {
            Some(entry) => entry,
            None => {
                warn!("No entry found for {}", name);
                return None;
            }
        };
        Some(entry.ip.clone())
    }

    pub async fn get_addr_of(&self, name: String) -> Option<String> {
        let ip = match self.get_ip_of(name.clone()).await {
            Some(ip) => ip,
            None => {
                warn!("No entry found for {}", name);
                return None;
            }
        };
        let port = match self.get_port_of(name.clone()).await {
            Some(port) => port,
            None => {
                warn!("No entry found for {}", name);
                return None;
            }
        };

        Some(format!("{}:{}", ip, port))
    }

    pub fn get_own_name(&self) -> String {
        self.own_name.clone()
    }

    pub async fn send_message(&mut self, target: String, message: String) {
        let routingtable = self.routingtable.lock().await;
        let entry = match routingtable.get_entry(target.clone()) {
            Some(entry) => entry,
            None => {
                warn!("No entry found for {}", target);
                return;
            }
        };
        let mut connection = TcpStream::connect(entry.get_address()).unwrap();
        let header = BaseHeader::new(
            PacketType::Message,
            32,
            entry.destination.clone(),
            self.own_name.clone(),
            entry.hops,
        );
        let mut msg = BytesMut::with_capacity(1024);
        msg.put(header.to_bytes());
        msg.put(message.as_bytes());
        let packet = Packet::create_crc32(msg).await;
        connection.write_all(&packet.to_bytes()).unwrap();
        connection.flush().unwrap();
        debug!("Sent message to {}", target);
    }
}
