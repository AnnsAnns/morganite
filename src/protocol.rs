use common_header::CommonHeader;
use routed_packet::RoutedPacket;
use routing_packet::RoutingPacket;

pub mod common_header;
pub mod routed_packet;
pub mod routing_packet;

#[derive(Clone)]
pub enum Packet {
    RoutedPacket(RoutedPacket),
    RoutingPacket(RoutingPacket),
}

pub const ROUTING_PACKET_TYPE: u8 = 1;
pub const ROUTED_PACKET_TYPE: u8 = 2;
