use std::fmt::{Formatter, self, Display};

use bytes::{BufMut, BytesMut};
use log::{debug, warn};


pub struct RoutingEntry {
    pub info_source: String,
    pub destination: String,
    pub ip: String,
    pub port: u16,
    pub hops: u8,
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

    pub fn get_address(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }

    pub fn to_bytes(&self) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(1024);
        bytes.put(self.info_source.as_bytes());
        bytes.put(self.destination.as_bytes());
        bytes.put_u16(self.port);
        bytes.put_u8(self.hops);
        bytes
    }
}

impl Display for RoutingEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "RoutingEntry {{ info_source: {}, destination: {}, ip: {}, port: {}, hops: {} }}", self.info_source, self.destination, self.ip, self.port, self.hops)
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
        self.entries.iter().find(|&entry| entry.destination == destination)
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
            if entry.info_source == poise {
                continue;
            }
            bytes.put(entry.to_bytes());
        }
        bytes
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