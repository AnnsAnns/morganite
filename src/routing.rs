use std::fmt::{self, Display, Formatter};

use bytes::{BufMut, BytesMut};
use log::{debug};

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
    pub fn add_entry(&mut self, entry: RoutingEntry) {
        debug!(
            "New \"{}\" from \"{}\" ({}:{})",
            &entry.destination, &entry.info_source, &entry.ip, &entry.port
        );
        self.entries.push(entry);
    }

    /**
     * Returns the entry with the given destination
     */
    pub fn get_entry(&self, destination: String) -> Option<&RoutingEntry> {
        self.entries
            .iter()
            .find(|&entry| entry.destination == destination)
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

    pub fn to_bytes(&self, poise: String) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(1024);
        bytes.put_u8(self.total_entries(poise.clone()) as u8);
        for entry in &self.entries {
            if entry.info_source == poise || entry.destination == poise {
                continue;
            }
            bytes.put(entry.to_bytes());
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
