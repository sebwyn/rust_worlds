use std::error::Error;

use reliable_udp::{Connection, ipv4_from_str};

#[allow(dead_code)]
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



fn main() -> Result<(), Box<dyn Error>>{
    let mut connection = Connection::<Message>::new(1234, ipv4_from_str("127.0.0.1")?, 1337)?;

    connection.fake_packet_loss(0.2);

    loop {
        //wait for input
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();

        if input == "quit" { break }

        connection.send_packet(Message::from(input.as_str()))?;
        std::thread::sleep(std::time::Duration::from_millis(5));
        
        //try to receive packets here
        connection.receive_packet()?;
    }

    Ok(())
}
