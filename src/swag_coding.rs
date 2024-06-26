use std::cmp::Ordering;

use tokio_util::{
    bytes::BytesMut,
    codec::{Decoder, Encoder},
};

use crate::protocol::{
    common_header::{CommonHeader, CommonHeaderUnparsed, COMMON_HEADER_LENGTH},
    routed_packet::RoutedPacket,
    routing_packet::RoutingPacket,
    Packet, CR, CRR, MESSAGE, SCC, SCCR, STU,
};

#[cfg(test)]
use crate::protocol::shared_header::SharedHeader;

// Swag Decoder is a custom decoder for the SWAG protocol
pub struct SwagCoder {
    has_common_header: bool,
    last_common_header: Option<CommonHeader>,
}

const MAX_ACCEPTED_LEN: usize = 8 * 1024 * 1024;

impl SwagCoder {
    pub fn new() -> Self {
        SwagCoder {
            has_common_header: false,
            last_common_header: None,
        }
    }
}

impl Decoder for SwagCoder {
    type Item = Packet;

    type Error = std::io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Check whether the buffer is too large
        if src.len() > MAX_ACCEPTED_LEN {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Buffer too large: {} - Exiting as a protection", src.len()),
            ));
        }

        // If the common header hasn't arrived yet we need to read it
        if !self.has_common_header {
            // Check whether we have enough bytes to read the common header
            if src.len() < COMMON_HEADER_LENGTH {
                return Ok(None);
            }

            let header_bytes = src.split_to(COMMON_HEADER_LENGTH);

            // serde deserialization
            let header_unparsed: CommonHeaderUnparsed = match serde_json::from_slice(&header_bytes)
            {
                Ok(header) => header,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Error deserializing common header: {}", e),
                    ));
                }
            };

            tracing::debug!("Received header unparsed: {:?}", header_unparsed);
            let header = CommonHeader::from_unparsed(header_unparsed);
            tracing::debug!("Received header: {:?}", header);

            self.last_common_header = Some(header);
            self.has_common_header = true;
            self.decode(src)
        } else {
            let header = self.last_common_header.unwrap();

            // Check whether we have enough bytes to read the packet
            let packet_length = header.length as usize;
            if src.len() < packet_length {
                return Ok(None);
            }

            let packet_bytes = src.split_to(packet_length);

            // Verify the checksum
            let checksum = crc32fast::hash(&packet_bytes);
            if checksum != header.crc32 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Checksum mismatch: {} != {}", checksum, header.crc32),
                ));
            }

            // Deserialize the packet
            let packet = match header.type_id {
                CR | CRR | SCC | SCCR | STU => {
                    let packet: RoutingPacket = match serde_json::from_slice(&packet_bytes) {
                        Ok(packet) => packet,
                        Err(e) => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Error deserializing routing packet: {}", e),
                            ));
                        }
                    };

                    self.has_common_header = false;
                    Packet::RoutingPacket(packet, header.type_id)
                }
                MESSAGE => {
                    let packet: RoutedPacket = match serde_json::from_slice(&packet_bytes) {
                        Ok(packet) => packet,
                        Err(e) => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Error deserializing routed packet: {}", e),
                            ));
                        }
                    };

                    self.has_common_header = false;
                    Packet::RoutedPacket(packet)
                }
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Unknown packet type: {}", header.type_id),
                    ));
                }
            };

            self.has_common_header = false;
            Ok(Some(packet))
        }
    }
}

impl Encoder<Packet> for SwagCoder {
    type Error = std::io::Error;

    fn encode(&mut self, item: Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let payload_bytes = match item.clone() {
            Packet::RoutingPacket(packet, _) => {
                tracing::debug!(
                    "Encoding routing packet: {:?}",
                    serde_json::to_string(&packet).unwrap()
                );

                match serde_json::to_vec(&packet) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Error serializing routing packet: {}", e),
                        ));
                    }
                }
            }
            Packet::RoutedPacket(packet) => match serde_json::to_vec(&packet) {
                Ok(bytes) => bytes,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Error serializing routed packet: {}", e),
                    ));
                }
            },
        };

        // Calculate the checksum
        tracing::debug!("Payload bytes: {:?}", payload_bytes.as_slice());
        let checksum = crc32fast::hash(&payload_bytes.as_slice());
        // Create the common header
        let header = CommonHeader {
            length: payload_bytes.len() as u16,
            crc32: checksum,
            type_id: match item {
                Packet::RoutingPacket(_, type_id) => type_id,
                Packet::RoutedPacket(_) => MESSAGE,
            },
        };
        let header_stringify = CommonHeaderUnparsed::new(header);
        let header_string = serde_json::to_string(&header_stringify).unwrap();
        let header_bytes = header_string.as_bytes();

        match header_bytes.len().cmp(&COMMON_HEADER_LENGTH) {
            Ordering::Less => {
                let mut new_header_bytes = [0; COMMON_HEADER_LENGTH];
                new_header_bytes[..header_bytes.len()].copy_from_slice(header_bytes);
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Common header too small: {} - Should be {}",
                        header_bytes.len(),
                        COMMON_HEADER_LENGTH
                    ),
                ));
            }
            Ordering::Greater => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!(
                        "Common header too large: {} - Should be {}",
                        header_bytes.len(),
                        COMMON_HEADER_LENGTH
                    ),
                ));
            }
            Ordering::Equal => {
                // Reserve space for the common header & packet
                dst.reserve(COMMON_HEADER_LENGTH + payload_bytes.len());

                // Write the common header
                dst.extend_from_slice(header_bytes);

                // Write the packet
                dst.extend_from_slice(&payload_bytes);
            }
        }

        Ok(())
    }
}

#[test]
pub fn test_weird_decode() {
    let mut coder = SwagCoder::new();
    let packet = RoutedPacket {
        header: SharedHeader {
            source_ip: "127.0.0.1".to_string(),
            source_port: 58471,
            dest_ip: "127.0.0.1".to_string(),
            dest_port: 6143,
            ttl: 16,
        },
        nickname: "TODO".to_string(),
        message: "hi".to_string(),
    };
    let mut encoded = BytesMut::new();
    coder
        .encode(Packet::RoutedPacket(packet), &mut encoded)
        .unwrap();
    let result = coder.decode(&mut encoded);
    println!("result: {:?}", result.unwrap());
}

#[test]
pub fn test_recoding() {
    let mut coder = SwagCoder::new();
    let original_packet = RoutedPacket {
        header: SharedHeader {
            source_ip: "127.0.0.1".to_string(),
            source_port: 58471,
            dest_ip: "127.0.0.1".to_string(),
            dest_port: 6143,
            ttl: 16,
        },
        nickname: "test_nickname".to_string(),
        message: "hello".to_string(),
    };

    // Encode the packet
    let mut encoded = BytesMut::new();
    coder
        .encode(Packet::RoutedPacket(original_packet.clone()), &mut encoded)
        .unwrap();

    // Decode the packet
    let decoded_packet = coder.decode(&mut encoded).unwrap().unwrap();

    // Check that the decoded packet matches the original packet
    if let Packet::RoutedPacket(decoded_routed_packet) = decoded_packet {
        assert_eq!(original_packet, decoded_routed_packet);
    } else {
        panic!("Decoded packet is not of type RoutedPacket");
    }

    // Ensure buffer is empty after successful decode
    assert!(encoded.is_empty());
}

#[test]
pub fn test_checksum_creation() {
    static EXPECTED_CHECKSUM: u32 = 593877371; // Generated via https://crc32.online/
    let example_data = b"{\"header\":{\"source_ip\":\"10.241.51.185\",\"source_port\":46455,\"dest_ip\":\"10.241.51.185\",\"dest_port\":50847,\"ttl\":16},\"table\":[]}";
    let checksum = crc32fast::hash(example_data);
    assert_eq!(checksum, EXPECTED_CHECKSUM);
}
