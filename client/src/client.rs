use std::{
    error::Error,
    net::{SocketAddr, UdpSocket}, mem::size_of,
};

use app::HandShake;
use networking::{ipv4_from_str, stream::Agent};
use serde::{Serialize, de::DeserializeOwned};

pub fn open_connection<S, R>(ip: &str) -> Result<Agent<S, R>, Box<dyn Error>> 
where
    S: Send + Sync + std::fmt::Debug + Serialize + 'static,
    R: Send + Sync + std::fmt::Debug + DeserializeOwned + 'static,
{
    let (hand_shake, port): (HandShake, u16) = {
        let local_address = SocketAddr::new(ipv4_from_str("127.0.0.1")?, 0);
        let hand_shake_socket = UdpSocket::bind(local_address)?;

        let server_address = SocketAddr::new(ipv4_from_str(ip)?, app::SERVER_ROUTER_PORT);
        hand_shake_socket.send_to(&[], server_address).unwrap();

        let mut hand_shake_buffer = [0u8; size_of::<app::HandShake>()];
        hand_shake_socket.recv_from(&mut hand_shake_buffer)?;

        let port = hand_shake_socket.local_addr()?.port();

        (
            app::deserialize(&hand_shake_buffer).expect("Couldn't deserialize handshake"),
            port,
        )
    };

    Agent::start(Some(port), ip, hand_shake.port)
}
