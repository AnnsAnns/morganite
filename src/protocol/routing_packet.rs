use std::array;

use serde::{Deserialize, Serialize};

use super::shared_header::SharedHeader;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct  RoutingEntry { //not sure about the int types here, we didn't specify anything in the protocol
    pub target_ip: String,
    pub target_port: i32,
    pub next: String,
    pub next_port: i32,
    pub hop_count: i32,
}

///IF I understood this correctly every routing packet looks like this and just has a different type_id to trigger a different reaction 
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoutingPacket {
    pub header: SharedHeader,
    pub table: Vec<RoutingEntry>, //not quite sure about the type of array here, haven't tried yet
}