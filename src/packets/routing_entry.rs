use std::fmt::{self, Display, Formatter};

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
    pub fn new(
        info_source: String,
        destination: String,
        ip: String,
        port: u16,
        hops: u8,
    ) -> RoutingEntry {
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

    pub fn from_bytes(bytes: BytesMut, ip: String) -> RoutingEntry {
        let info_source = String::from_utf8(bytes[0..3].to_vec()).unwrap();
        let destination = String::from_utf8(bytes[3..6].to_vec()).unwrap();
        let port = u16::from_be_bytes([bytes[6], bytes[7]]);
        let hops = bytes[8];

        RoutingEntry {
            info_source,
            destination,
            ip,
            port,
            hops,
        }
    }
}

impl Display for RoutingEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "RoutingEntry {{ info_source: {}, destination: {}, ip: {}, port: {}, hops: {} }}",
            self.info_source, self.destination, self.ip, self.port, self.hops
        )
    }
}