use std::fmt::{self, Display, Formatter};

use bytes::{BufMut, BytesMut};
use log::debug;

use crate::packets::routing_entry::RoutingEntry;

pub struct Routingtable {
    entries: Vec<RoutingEntry>,
}

impl Routingtable {
    pub fn new() -> Routingtable {
        Routingtable {
            entries: Vec::new(),
        }
    }

    /**
     * Removes all entries from the given source
     */
    pub fn clear_from(&mut self, poise: String) {
        self.entries.retain(|entry| entry.info_source != poise);
    }

    /**
     * Adds a new entry to the routing table
     */
    pub async fn add_entry(&mut self, entry: RoutingEntry) {
        debug!(
            "New Entry \"{}\" from \"{}\" ({}:{})",
            &entry.destination, &entry.info_source, &entry.ip, &entry.port
        );
        self.entries.push(entry);
    }

    pub async fn get_direct_sources(&self, of_target: String) -> Vec<String> {
        let mut direct_sources: Vec<String> = Vec::new();
        for entry in &self.entries {
            if entry.info_source == of_target {
                direct_sources.push(entry.destination.clone());
            }
        }
        direct_sources
    }

    /**
     * Returns the entry with the given destination
     */
    pub fn get_entry(&self, destination: String) -> Option<&RoutingEntry> {
        self.entries
            .iter()
            .find(|&entry| entry.destination == destination)
    }

    pub async fn remove_entry(&mut self, destination: String) {
        self.entries.retain(|entry| {
            entry.destination != destination
        });
    }

    pub fn total_entries(&self, poise: String) -> usize {
        let mut total = 0;
        for entry in &self.entries {
            if entry.info_source == poise {
                continue;
            }
            total += 1;
        }
        total
    }

    pub fn to_bytes(&self, poise: String, own_ip: String, own_port: u16, own_name: String) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(1024);
        bytes.put_u8(self.total_entries(poise.clone()) as u8);
        for entry in &self.entries {
            if entry.info_source == poise || entry.destination == poise {
                continue;
            }
            let translated_entry = RoutingEntry::new(
                own_name.clone(),
                entry.destination.clone(),
                own_ip.clone(),
                own_port.clone(),
                entry.hops+1, // Since we are the next hop we have to increase the hops
            );
            debug!("Sending routing entry for {}", translated_entry.destination);
            debug!("IP: {}:{}", translated_entry.ip, translated_entry.port);
            debug!("Hops: {}", translated_entry.hops);
            bytes.put(translated_entry.to_bytes());
        }
        bytes
    }

    pub fn get_entries(&self) -> &Vec<RoutingEntry> {
        &self.entries
    }
}

impl Display for Routingtable {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Routingtable:")?;
        for entry in &self.entries {
            writeln!(f, "{}", entry)?;
        }

        Ok(())
    }
}

impl Default for Routingtable {
    fn default() -> Self {
        Self::new()
    }
}
