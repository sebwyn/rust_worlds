use serde::{Serialize, Deserialize};

pub const SERVER_ROUTER_PORT: u16 = 6669;
pub const MAX_PACKET_SIZE: usize = 1000; //a kilobyte

pub fn serialize<T>(value: T) -> [u8; MAX_PACKET_SIZE]
where
    T: Serialize
{
    //this will be slow as shit, a kilobyte allocation all the time, very cringe, idk kb isn't that
    //big
    let mut buffer = [0u8; MAX_PACKET_SIZE];
    ciborium::ser::into_writer(&value, buffer.as_mut_slice()).expect("Exceeded max packet size??");
    buffer
}

pub fn deserialize<'a, T>(buffer: &[u8]) -> Option<T>
where
    T: Deserialize<'a>
{
    ciborium::de::from_reader(buffer).ok()
}

#[derive(Serialize, Deserialize)]
pub struct HandShake {
    pub port: u16,
}

impl HandShake {
    pub fn new(port: u16) -> Self {
        Self {
            port
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Message {
    pub bytes: [u8; 100]
}

impl Default for Message {
    fn default() -> Self {
        Self { bytes: [0u8; 100] }
    }
}

impl From<&str> for Message {
    fn from(message: &str) -> Self {
        let mut bytes = [0u8; 100];
        bytes[..message.len()].copy_from_slice(message.as_bytes());

        Self {
            bytes
        }
    }
}
