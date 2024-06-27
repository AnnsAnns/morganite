
use routed_packet::RoutedPacket;
use routing_packet::RoutingPacket;

pub mod common_header;
pub mod routed_packet;
pub mod routing_packet;
pub mod shared_header;

#[derive(Clone, Debug)]
pub enum Packet {
    RoutedPacket(RoutedPacket),
    RoutingPacket(RoutingPacket, u8),
}

pub const MESSAGE: u8 = 1;
pub const CR: u8 = 2;
pub const CRR: u8 = 3;
pub const SCC: u8 = 4;
pub const SCCR: u8 = 5;
pub const STU: u8 = 6;

