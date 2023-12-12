
use bytes::{BytesMut, BufMut};
use log::{info, debug, warn};

use std::collections::HashSet;
use std::io::Write;
use std::net::TcpStream;
use std::sync::Arc;
use tokio::sync::Mutex;


use crate::{RoutingTableType, routing::Routingtable, packets::{routing_entry::RoutingEntry, header::{BaseHeader, BASE_HEADER_SIZE}}};


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
        self.routingtable.lock().await.add_entry(entry)
    }

    /**
     * Broadcasts the routing table to all direct sources
     */
    pub async fn broadcast_routingtable(&mut self) {
        //debug!("Broadcasting routingtable");
        let mut direct_sources: HashSet<String> = HashSet::new();
        for sources in self.routingtable.lock().await.get_entries() {
            direct_sources.insert(sources.info_source.clone());
        }   

        for entry in direct_sources {
            self.send_routingtable(entry.clone()).await;
        }
    }

    /**
     * Sends the routing table to the given destination
     * @param destination The destination to send the routing table to (name)
     */
    pub async fn send_routingtable(&mut self, destination: String) {
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
            0, 
            32,
            entry.destination.clone(),
            self.own_name.clone(),
            entry.hops,
        ).to_bytes();

        let routingtable_bytes = routingtable.to_bytes(entry.ip.clone());

        debug!("Sending routingtable to {}", destination);
        let mut msg = BytesMut::with_capacity(1024);
        msg.put(header.clone());
        msg.put(routingtable_bytes.clone());

        let mut connection = TcpStream::connect(entry.get_address()).unwrap();
        connection.write_all(&msg).unwrap();
        connection.flush().unwrap();
        debug!("Sent routingtable to {}", destination);
    }

    /**
     * Creates a new CRC32 checksum from the given bytes
     */
    pub async fn create_crc32(&mut self, bytes: BytesMut) -> u32 {
        let mut crc32 = crc32fast::Hasher::new();
        crc32.update(&bytes);
        crc32.finalize()
    }

    /**
     * Verifies the given CRC32 checksum against the given bytes
     */
    pub async fn verify_crc32(&mut self, bytes: BytesMut, checksum: u32) -> bool {
        let mut crc32 = crc32fast::Hasher::new();
        crc32.update(&bytes);
        crc32.finalize() == checksum
    }

    /**
     * Updates the routing table with the given bytes
     * @param bytes The bytes to update the routing table with
     * @param ip The ip of the sender
     */
    pub async fn update_routing_table(&mut self, bytes: BytesMut, _ip: String) {
        let header = BaseHeader::from_bytes(bytes.clone());
        self.routingtable.lock().await.clear_from(header.get_ip()); // clear all entries from the source
        let routingtable_bytes = bytes[BASE_HEADER_SIZE..].to_vec();
        let total_entries = routingtable_bytes[0];
        let mut offset = 1;
        for _ in 0..total_entries {
            let entry_bytes = routingtable_bytes[offset..offset+9].to_vec();
            let entry = RoutingEntry::from_bytes(BytesMut::from(entry_bytes.as_slice()),
                                                 header.get_ip());
            if entry.info_source == self.own_name || entry.destination == self.own_name || entry.destination == entry.info_source {
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