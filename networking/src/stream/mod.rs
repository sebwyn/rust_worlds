const MAX_PACKET_SIZE: usize = 2048; 

mod agent;
pub use agent::Agent;

#[cfg(test)]
mod agent_test;

mod connection;
use connection::Connection;

#[cfg(test)]
mod connection_test;

