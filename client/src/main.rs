use std::error::Error;

use reliable_udp::{Connection, ipv4_from_str};

fn main() -> Result<(), Box<dyn Error>>{
    let mut connection = Connection::new(1234, ipv4_from_str("127.0.0.1")?, 1337)?;

    for _ in 0..10 {
        connection.send_packet()?;
        
        //if connection.poll_packet()? {
            connection.receive_packets()?;
        //}
    }

    Ok(())
}
