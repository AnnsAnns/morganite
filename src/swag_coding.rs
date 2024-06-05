use tokio_util::{bytes::{Buf, BytesMut}, codec::{Decoder, Encoder}};

use crate::protocol::{common_header::{CommonHeader, COMMON_HEADER_LENGTH}, routed_packet::RoutedPacket, routing_packet::RoutingPacket, Packet};

// Swag Decoder is a custom decoder for the SWAG protocol
struct SwagCoder {
    has_common_header: bool,
    last_common_header: Option<CommonHeader>,
}

const MAX_ACCEPTED_LEN: usize = 8 * 1024 * 1024;

impl Decoder for SwagCoder {
    type Item = Packet;

    type Error = std::io::Error;

    fn decode(&mut self, src: &mut tokio_util::bytes::BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Check whether the buffer is too large
        if src.len() > MAX_ACCEPTED_LEN {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Buffer too large: {} - Exiting as a protection", src.len())
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
            let header: CommonHeader = match serde_json::from_slice(&header_bytes) {
                Ok(header) => header,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Error deserializing common header: {}", e)
                    ));
                }
            };

            self.last_common_header = Some(header);
            self.has_common_header = true;
        } else {
            let header = self.last_common_header.unwrap();

            // Check whether we have enough bytes to read the packet
            let packet_length = header.header.length as usize;
            if src.len() < packet_length {
                return Ok(None);
            }

            let packet_bytes = src.split_to(packet_length);

            // Verify the checksum
            let checksum = crc32fast::hash(&packet_bytes);
            if checksum != header.header.crc32 {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Checksum mismatch: {} != {}", checksum, header.header.crc32)
                ));
            }

            // Deserialize the packet
            let packet = match header.header.type_id {
                ROUTING_PACKET_TYPE => {
                    let packet: RoutingPacket = match serde_json::from_slice(&packet_bytes) {
                        Ok(packet) => packet,
                        Err(e) => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Error deserializing routing packet: {}", e)
                            ));
                        }
                    };

                    self.has_common_header = false;
                    Packet::RoutingPacket(packet)
                },
                ROUTED_PACKET_TYPE => {
                    let packet: RoutedPacket = match serde_json::from_slice(&packet_bytes) {
                        Ok(packet) => packet,
                        Err(e) => {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("Error deserializing routed packet: {}", e)
                            ));
                        }
                    };

                    self.has_common_header = false;
                    Packet::RoutedPacket(packet)
                },
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Unknown packet type: {}", header.header.type_id)
                    ));
                }
            };

            self.has_common_header = false;
            return Ok(Some(packet));
        }

        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Error processing packet")
        ));;
    }
}

impl Encoder<Packet> for SwagCoder {
    type Error = std::io::Error;
    
    fn encode(&mut self, item: Packet, dst: &mut BytesMut) -> Result<(), Self::Error> {


        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Not implemented")
        ));
    }
}