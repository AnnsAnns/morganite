use bytes::{BufMut, BytesMut};
use log::{debug, error, info, trace, warn};

pub struct RoutingEntry {
    info_source: String,
    destination: String,
    ip: String,
    port: u16,
    hops: u8,
}

impl RoutingEntry {
    pub fn new(info_source: String, destination: String, ip: String, port: u16, hops: u8) -> RoutingEntry {
        let mut truncated_source = info_source.clone();
        if info_source.len() > 3 {
            warn!("Info source address is too long, truncating to 3 characters");
            truncated_source.truncate(3);
        }

        let mut truncated_destination = destination.clone();
        if destination.len() > 3 {
            warn!("Destination address is too long, truncating to 3 characters");
            truncated_destination.truncate(3);
        }

        RoutingEntry {
            info_source: truncated_source,
            destination: truncated_destination,
            ip,
            port,
            hops,
        }
    }
}

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
     * Adds a new entry to the routing table
     */
    pub fn add_entry(&mut self, entry: RoutingEntry) {
        debug!("New \"{}\" from \"{}\" ({}:{})", &entry.destination, &entry.info_source, &entry.ip, &entry.port);
        self.entries.push(entry);
    }

    /**
     * Returns the entry with the given destination
     */
    pub fn get_entry(&self, destination: String) -> Option<&RoutingEntry> {
        for entry in &self.entries {
            if entry.destination == destination {
                return Some(entry);
            }
        }
        None
    }


}