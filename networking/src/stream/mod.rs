const MAX_PACKET_SIZE: usize = 2048; 
const MAX_PAYLOAD_SIZE: usize = MAX_PACKET_SIZE - std::mem::size_of::<connection::Header>();

mod agent;
pub use agent::Agent;

#[cfg(test)]
mod agent_test;

mod connection;
use connection::Connection;

#[cfg(test)]
mod connection_test;

