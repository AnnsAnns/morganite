use std::fmt::{self, Display, Formatter};

use bytes::{BufMut, BytesMut};
use log::{warn, debug};

#[derive(Debug)]
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
        debug!("Byte Translation: {} to {:#?}", self.info_source.clone(), self.info_source.clone().as_bytes());
        bytes.put(self.info_source.as_bytes());
        debug!("Byte Translation: {} to {:#?}", self.destination.clone(), self.destination.clone().as_bytes());
        bytes.put(self.destination.as_bytes());
        debug!("Byte Translation: {} to {:#?}", self.port.clone(), self.port.clone().to_be_bytes());
        bytes.put_u16(self.port);
        debug!("Byte Translation: {} to {:#?}", self.hops.clone(), self.hops.clone());
        bytes.put_u8(self.hops);
        bytes
    }

    pub fn from_bytes(bytes: BytesMut, ip: String) -> RoutingEntry {
        let info_source = String::from_utf8(bytes[0..3].to_vec()).unwrap();
        let destination = String::from_utf8(bytes[3..6].to_vec()).unwrap();
        let port = u16::from_be_bytes([bytes[6], bytes[7]]);
        let hops = bytes[8];

        debug!("Received routing entry for {}", destination);
        debug!("IP: {}:{}", ip, port);
        debug!("Hops: {}", hops);

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

#[cfg(test)]
mod tests {
    use bytes::BytesMut;

    use super::RoutingEntry;

    #[test]
    fn to_bytes() {
        let entry = RoutingEntry {
            info_source: String::from("ABC"),
            destination: String::from("DEF"),
            ip: String::from("127.0.0.1"),
            port: 1234,
            hops: 0,
        };

        let bytes = entry.to_bytes();
        // ABC
        assert_eq!(bytes[0], 65);
        assert_eq!(bytes[1], 66);
        assert_eq!(bytes[2], 67);
        // DEF
        assert_eq!(bytes[3], 68);
        assert_eq!(bytes[4], 69);
        assert_eq!(bytes[5], 70);
        // 1234 = 0x04D2
        assert_eq!(bytes[6], 4);
        assert_eq!(bytes[7], 210);
        // 0
        assert_eq!(bytes[8], 0);
    }

    #[test]
    fn from_bytes() {
        let bytes = vec![65, 66, 67, 68, 69, 70, 4, 210, 0];
        let bytes = BytesMut::from(bytes.as_slice());
        let entry = RoutingEntry::from_bytes(bytes, String::from("127.0.0.1"));

        assert_eq!(entry.info_source, String::from("ABC"));
        assert_eq!(entry.destination, String::from("DEF"));
        assert_eq!(entry.ip, String::from("127.0.0.1"));
        assert_eq!(entry.port, 1234);
        assert_eq!(entry.hops, 0);
    }

    #[test]
    fn to_bytes_and_back() {
        let entry = RoutingEntry {
            info_source: String::from("ABC"),
            destination: String::from("DEF"),
            ip: String::from("127.0.0.1"),
            port: 1234,
            hops: 0,
        };

        let bytes = entry.to_bytes();
        let entry = RoutingEntry::from_bytes(bytes, String::from("127.0.0.1"));

        assert_eq!(entry.info_source, String::from("ABC"));
        assert_eq!(entry.destination, String::from("DEF"));
        assert_eq!(entry.ip, String::from("127.0.0.1"));
        assert_eq!(entry.port, 1234);
        assert_eq!(entry.hops, 0);
    }
}