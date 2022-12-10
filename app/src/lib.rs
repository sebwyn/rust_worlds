use serde::{Serialize, Deserialize};

pub const SERVER_ROUTER_PORT: u16 = 6669;
pub const MAX_PACKET_SIZE: usize = 2000; //a kilobyte

mod serialization;
pub use serialization::{serialize, deserialize, serialized_size};

mod client_event;
pub use client_event::ClientEvent;

#[derive(Serialize, Deserialize)]
pub struct HandShake {
    pub port: u16,
}

impl HandShake {
    pub fn new(port: u16) -> Self {
        Self { port }
    }
}

struct Sphere { 
    position: [f32; 3],
    color: [f32; 4],
    radius: f32,
}

struct World(Vec<Sphere>);

mod hash_packet;
pub use hash_packet::{HashPacketEncoder, HashPacketDecoder};

#[cfg(test)]
mod hash_packet_test;
