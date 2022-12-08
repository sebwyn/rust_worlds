use std::error::Error;
use reliable_udp::{Connection, ipv4_from_str};

#[derive(Clone, Copy, Debug)]
struct Message {
    bytes: [u8; 100]
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

fn main() -> Result<(), Box<dyn Error>> {

    //create a connection
    let mut connection = Connection::<Message>::new(1337, ipv4_from_str("127.0.0.1")?, 1234)?;

    loop {
        if let Some(message) = connection.receive_packet()? {
            //decode the message here
            let message = String::from_utf8(message.bytes.to_vec())?;
            println!("{}", message);

            //ack packets manually 
            connection.send_packet(Message::from(""))?;
        }
        //println!("Received a message from {}:{}: {}", src.ip(), src.port(), message);
    }
}
