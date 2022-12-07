use std::{net::UdpSocket, error::Error};

use reliable_udp::{Connection, ipv4_from_str};

fn main() -> Result<(), Box<dyn Error>> {

    //create a connection
    let mut connection = Connection::new(1337, ipv4_from_str("127.0.0.1")?, 1234)?;

    loop {
        if connection.receive_packets()? {
            //ack packets manually 
            connection.send_packet()?;
        }
        //println!("Received a message from {}:{}: {}", src.ip(), src.port(), message);
    }
}
