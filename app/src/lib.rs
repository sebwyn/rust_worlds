pub const SERVER_ROUTER_PORT: u16 = 6669;
pub const MAX_PACKET_SIZE: usize = 2000; //a kilobyte

mod serialization;
pub use serialization::{serialize, deserialize, serialized_size};

mod packets;
pub use packets::{GameObject, ClientEvent, Snapshot, HandShake, Transform};

pub mod components;

mod hash_packet;
pub use hash_packet::{HashPacketEncoder, HashPacketDecoder};

#[cfg(test)]
mod hash_packet_test;
