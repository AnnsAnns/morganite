use std::array;

use serde::{Deserialize, Serialize};

use super::shared_header::SharedHeader;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct  RoutingTableEntry {
    target: String,
    next: String,
    hop_count: String,
}

///IF I understood this correctly every routing packet looks like this and just has a different type_id to trigger a different reaction 
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoutingPacket {
    header: SharedHeader,
    data: String, //contains the name?
    table: Vec<RoutingTableEntry>, //not quite sure about the type of array here, haven't tried yet
}