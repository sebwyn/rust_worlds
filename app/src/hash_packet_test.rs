use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::{HashPacketEncoder, HashPacketDecoder};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Position {
    name: String,
    y: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AnotherStruct {
    x: u32,
    y: u32,
    z: i32,
}

#[test]
fn test_packets() {
    let positions = vec![Position { name: "Hello".to_string(), y: 0}; 10];

    let mut encoder = HashPacketEncoder::new();
    let added = encoder.add_element("positions", positions.clone());
    assert!(added, "Failed to add a valid struct to packet");
    let bytes = encoder.submit();

    let mut decoder = HashPacketDecoder::decode(bytes);
    let out_positions = decoder.take_element::<Vec<Position>>("positions");
    assert!(out_positions.is_some(), "Failed to get positions back!");
    assert_eq!(out_positions.unwrap().len(), positions.len(), "Positions length didn't remain the same!");

    println!("{:#?}", positions);
}
