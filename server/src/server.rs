use std::{error::Error,  net::{SocketAddr, UdpSocket}, thread::JoinHandle};
use reliable_udp::Connection;

use app::{Message, HandShake};

//think about tokio for dealing with all these threads
#[derive(Default)]
pub struct Server {
    connections: Vec<(SocketAddr, JoinHandle<()>)>,
}

impl Server {
    //think about using tcp for routing, and handshaking
    pub fn run(&mut self) -> Result<(), Box<dyn Error>>  {

        let router = UdpSocket::bind("127.0.0.1:6669")?;

        //I'm assuming each send gets mapped to one recv, but idk, who knows, we'll find out
        let mut big_packet_buffer = vec![0u8; app::MAX_PACKET_SIZE];

        //this will be a blocking recv_from
        loop {
            let (_packet, client) = router.recv_from(&mut big_packet_buffer).expect("The server failed to receive? Dead??");

            if let Ok((connection, port)) = Connection::<app::Message>::new(client, None) {
                let hand_shake = HandShake::new(port);
                //send this client a packet back with the port of the connection
                router.send_to(app::serialize(hand_shake).as_slice(), client).expect("Failed to send hand_shake??");

                println!("Creating a thread to handle this: {:?}", connection);
                let join_handle = std::thread::spawn(move || { 
                    Self::maintain_connection(connection) 
                });

                //create a connection and spawn a thread to manage it
                self.connections.push((client, join_handle))
            }
        }

    }

    pub fn maintain_connection(mut connection: Connection<Message>) {
        println!("Maintaining a connection with: {}", connection.client_address());

        loop {
            let message = match connection.receive_packet() {
                Ok(m) => m,
                Err(_) => break, //kill this thread if we had a receive error
            };
            if let Some(message) = message {
                //decode the message here
                let string_message = match String::from_utf8(message.bytes.to_vec()) {
                    Ok(sm) => sm,
                    Err(_) => continue,
                };
                println!("{}", string_message);

                //ack packets manually, if we fail to send on the connection also quit, we got
                //disconnected
                //connection.send_packet(Message::from(""))?;
                match connection.send_packet(Message::from("")) {
                    Ok(m) => m,
                    Err(_) => break, //kill this thread if we had a receive error
                };
            }
        }
    }
}
