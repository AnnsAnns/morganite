use common_header::CommonHeader;
use routed_packet::RoutedPacket;
use routing_packet::RoutingPacket;

pub mod common_header;
pub mod routed_packet;
pub mod routing_packet;
pub mod shared_header;

#[derive(Clone, Debug)]
pub enum Packet {
    RoutedPacket(RoutedPacket),
    RoutingPacket(RoutingPacket),
}

pub const ROUTING_PACKET_TYPE: u8 = 1;
pub const ROUTED_PACKET_TYPE: u8 = 2;

pub enum TypeID { //lots of new types to integrate, but only 2 different packet styles if I understood them correctly
    Message = 1,
    CR,
    CRR,
    SCC,
    SCCR,
    STU,
}