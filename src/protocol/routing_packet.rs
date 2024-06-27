use std::array;

use serde::{Deserialize, Serialize};

use super::shared_header::SharedHeader;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct  RoutingEntry { //not sure about the int types here, we didn't specify anything in the protocol
    pub target_ip: String,
    pub target_port: u16,
    pub next_ip: String,
    pub next_port: u16,
    pub hop_count: i32,
}

///IF I understood this correctly every routing packet looks like this and just has a different type_id to trigger a different reaction 
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoutingPacket {
    pub header: SharedHeader,
    pub table: Option<Vec<RoutingEntry>>, //works perfectly like this 
}

#[test]
fn test_parsing_routing_packet() {
    let json = r#"{"header":{
        "packet_type":       "data_message",
        "source_ip":          "192.168.123.122",
        "source_port":        "6827",
        "dest_ip":     "192.168.234.233",
        "dest_port":   "234",
        "ttl":                16},
        "table":           [
            {
              "target_ip": "10.0.0.5",
              "target_port": 1234,
              "next_ip": "10.0.0.3",
              "next_port": 1234,
              "hop_count": 4
            },
            {
              "target_ip": "10.0.0.11",
              "target_port": 1234,
              "next_ip": "10.0.0.6",
              "next_port": 1234,
              "hop_count": 2
            }
        ]
    }"#;
    let packet: RoutingPacket = serde_json::from_str(json).unwrap();
    assert_eq!(packet.header.source_ip, "192.168.123.122");
    assert_eq!(packet.header.source_port, 6827);
    assert_eq!(packet.header.dest_ip, "192.168.234.233");
    assert_eq!(packet.header.dest_port, 234);
    assert_eq!(packet.header.ttl, 16);
    assert_eq!(packet.table, Some(vec![RoutingEntry
        {
          target_ip: "10.0.0.5".to_string(),
          target_port: 1234,
          next_ip: "10.0.0.3".to_string(),
          next_port: 1234,
          hop_count: 4
        },
        RoutingEntry {
          target_ip: "10.0.0.11".to_string(),
          target_port: 1234,
          next_ip: "10.0.0.6".to_string(),
          next_port: 1234,
          hop_count: 2
        }
    ]));

}

#[test]
fn test_serializing_routing_packet() {
    let table = vec![RoutingEntry
    {
      target_ip: "10.0.0.5".to_string(),
      target_port: 1234,
      next_ip: "10.0.0.3".to_string(),
      next_port: 1234,
      hop_count: 4
    },
    RoutingEntry {
      target_ip: "10.0.0.11".to_string(),
      target_port: 1234,
      next_ip: "10.0.0.6".to_string(),
      next_port: 1234,
      hop_count: 2
    }];
    let packet = RoutingPacket {
        header: SharedHeader {source_ip: "192.168.101.101".to_string(), source_port: 1234, dest_ip: "153.132.143.121".to_string(), dest_port: 4321, ttl: 32},
        table: Some(table),
    };
    let json = serde_json::to_string(&packet).unwrap();

    assert_eq!(
        json,
        r#"{"header":{"source_ip":"192.168.101.101","source_port":"1234","dest_ip":"153.132.143.121","dest_port":"4321","ttl":32},"table":[{"target_ip":"10.0.0.5","target_port":1234,"next_ip":"10.0.0.3","next_port":1234,"hop_count":4},{"target_ip":"10.0.0.11","target_port":1234,"next_ip":"10.0.0.6","next_port":1234,"hop_count":2}]}"#
    );
}