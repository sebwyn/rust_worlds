mod serialization;
use serialization::{serialize, deserialize, serialized_size};

const MAX_PACKET_SIZE: usize = 2048; 
const MAX_PAYLOAD_SIZE: usize = MAX_PACKET_SIZE - std::mem::size_of::<connection::Header>();

mod connection;
pub use connection::{Connection, ipv4_from_str};

mod boof_socket;
pub use boof_socket::BoofSocket;

mod agent;
pub use agent::Agent;

#[cfg(test)]
mod agent_test;
