//this is essentially a test of clients talking to servers, the actual client will be far more
//complex
use std::{
    error::Error,
    net::{SocketAddr, UdpSocket}, mem::size_of, sync::mpsc,
};

use app::HandShake;

use reliable_udp::{ipv4_from_str, Connection};

pub fn open_connection(ip: &str) -> Result<(), Box<dyn Error>> {
    //open a connection
    let local_address = SocketAddr::new(ipv4_from_str("127.0.0.1")?, 0);
    let mut server_address = SocketAddr::new(ipv4_from_str(ip)?, app::SERVER_ROUTER_PORT);

    let (hand_shake, port): (HandShake, u16) = {
        let hand_shake_socket = UdpSocket::bind(local_address)?;
        hand_shake_socket.send_to(&[], server_address).unwrap();

        let mut hand_shake_buffer = [0u8; size_of::<app::HandShake>()];
        hand_shake_socket.recv_from(&mut hand_shake_buffer)?;

        let port = hand_shake_socket.local_addr()?.port();

        (
            app::deserialize(&hand_shake_buffer).expect("Couldn't deserialize handshake"),
            port,
        )
    };

    server_address.set_port(hand_shake.port);
    let (mut connection, _) = Connection::new(server_address, Some(port))?;
    //connection.fake_packet_loss(0.2);

    let (tx, rx) = mpsc::channel::<String>();

    std::thread::spawn(move || {
        loop {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            if input == "quit" {
                break;
            }

            let _result = tx.send(input);
        }
    });

    loop {
        //wait for input
        if let Ok(input) = rx.try_recv() {
            connection.send_bytes(&app::serialize(&input).unwrap())?;
        }
        //try to receive packets here
        if let Some(bytes) = connection.receive_bytes()? {
            //println!("Received bytes: {:?}", bytes);
            if let Some(messages) = app::deserialize::<Vec<String>>(&bytes) {
                //print all the messages we received
                for message in messages {
                    print!("{}", message);
                }
            }
        }

        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    //Ok(())
}
