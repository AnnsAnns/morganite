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

    pub fn to_bytes(&self, poise: String, _own_ip: String, _own_port: u16, _own_name: String) -> BytesMut {
        let mut bytes = BytesMut::with_capacity(1024);
        bytes.put_u8(self.total_entries(poise.clone()) as u8);
        for entry in &self.entries {
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_routingtable() {
//         let mut routingtable = Routingtable::new();
//         let entry = RoutingEntry::new(
//             String::from("ABC"),
//             String::from("DEF"),
//             String::from("127.0.0.1"),
//             1234,
//             0,
//         );
//         routingtable.add_entry(entry);
//         let entry = RoutingEntry::new(
//             String::from("ABC"),
//             String::from("GHI"),
//             String::from("127.0.0.2"),
//             1234,
//             0,
//         );

//         routingtable.add_entry(entry).await;
//         let entry = RoutingEntry::new(
//             String::from("DEF"),
//             String::from("GHI"),
//             String::from("127.0.0.3"),
//             1234,
//             0,
//         );

//         assert_eq!(routingtable.get_entry(String::from("DEF")).unwrap().ip, String::from("127.0.0.1"));
//         assert_eq!(routingtable.get_entry(String::from("GHI")).unwrap().ip, String::from("127.0.0.2"));
//         assert_eq!(routingtable.get_entry(String::from("GHI")).unwrap().port, 1234);
//         assert_eq!(routingtable.get_entry(String::from("GHI")).unwrap().hops, 0);
//         assert_eq!(routingtable.get_entry(String::from("GHI")).unwrap().destination, String::from("GHI"));
//         assert_eq!(routingtable.get_entry(String::from("GHI")).unwrap().info_source, String::from("ABC"));
//         assert_eq!(routingtable.get_entry(String::from("GHI")).unwrap().to_string(), String::from("RoutingEntry { info_source: ABC, destination: GHI, ip: 127.0.0.1, port: 1234, hops: 0 }"));
//         assert_eq!(routingtable.get_entry(String::from("GHI")).unwrap().to_bytes(), vec![65, 66, 67, 68, 69, 70, 4, 210, 0]);
//         assert_eq!(routingtable.get_entry(String::from("GHI")).unwrap().to_bytes().len(), 9);

//     }
// }