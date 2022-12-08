//this is essentially a test of clients talking to servers, the actual client will be far more
//complex
use std::{error::Error, net::{SocketAddr, UdpSocket}};

use app::{Message, HandShake};

use reliable_udp::{Connection, ipv4_from_str};

fn main() -> Result<(), Box<dyn Error>>{
    //open a connection
    let local_address = SocketAddr::new(ipv4_from_str("127.0.0.1")?, 0);
    let mut server_address = SocketAddr::new(ipv4_from_str("127.0.0.1")?, app::SERVER_ROUTER_PORT);
    
    let (hand_shake, port): (HandShake, u16) = {
        let hand_shake_socket = UdpSocket::bind(local_address)?;
        hand_shake_socket.send_to(&Message::from("").bytes, server_address)?;

        let mut hand_shake_buffer = [0u8; app::MAX_PACKET_SIZE];
        hand_shake_socket.recv_from(&mut hand_shake_buffer)?;
        
        let port = hand_shake_socket.local_addr()?.port();

        (app::deserialize(&hand_shake_buffer).expect("Couldn't deserialize handshake"), port)
    };
    
    server_address.set_port(hand_shake.port);
    let (mut connection, _) = Connection::<Message>::new(server_address, Some(port))?;
    //connection.fake_packet_loss(0.2);

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
