use std::{error::Error,  net::{SocketAddr, UdpSocket}, sync::{Arc, Mutex}, time::Duration};
use reliable_udp::Agent;

use app::HandShake;

//think about tokio for dealing with all these threads
pub struct Server {
    connections: Arc<Mutex<Vec<(SocketAddr, Agent<Vec<String>, String>)>>>,
    tick_rate: u64,
}

impl Server {
    pub fn new(tick_rate: u64) -> Self {
        Self {
            connections: Arc::new(Mutex::new(Vec::new())),
            tick_rate 
        } 
    }

    //think about using tcp for routing, and handshaking
    pub fn run(&mut self) -> Result<(), Box<dyn Error>>  {
        {
            let connections = self.connections.clone();
            std::thread::spawn(|| { Self::listen_thread(connections) });
        }

        let wait_time = 1000u64 / self.tick_rate; //in milliseconds
        
        loop { 
            self.close_stale_connections();
            let mut new_messages: Vec<(SocketAddr, String)> = self.get_client_messages(); 

            //update our world (modify our messages, so they look better)
            new_messages = new_messages.into_iter().map(|(addr, message)| {
                (
                    addr, 
                    format!("[{:?}]: {}", addr.to_string(), message)
                )
            }).collect();
             
            
            self.send_clients_messages(new_messages);

            std::thread::sleep(Duration::from_millis(wait_time));
        }

        //Ok(())
    }

    fn close_stale_connections(&mut self) {
        let mut clients = self.connections.lock().unwrap();

        clients.retain(|(_addr, agent)| {
            !agent.lost_connection()
        });
    }

    //loop and update our clients based on input
    fn get_client_messages(&mut self) -> Vec<(SocketAddr, String)> {
        let clients = self.connections.lock().unwrap();

        clients.iter().map(|client| {
            client.1.get_messages().into_iter().map(|m| (client.0, m))
        }).flatten().collect()
    }

    //send a message to our clients with the updated world state 
    fn send_clients_messages(&self, messages: Vec<(SocketAddr, String)>) {
        let clients = self.connections.lock().unwrap();

        for client in clients.iter() {
            //send a vector of messages excluding messages from this client
            let others_messages: Vec<String> = messages.iter().filter_map(|(addr, message)| {
                if *addr != client.0 { Some(message.clone()) } else { None }
            }).collect();

            if others_messages.len() > 0 {
                client.1.send_message(others_messages);
            }
        }
}

    fn listen_thread(connections: Arc<Mutex<Vec<(SocketAddr, Agent<Vec<String>, String>)>>>) {
        let router = UdpSocket::bind("127.0.0.1:6669").expect("Failed to open listen thread");

        //I'm assuming each send gets mapped to one recv, but idk, who knows, we'll find out
        let mut big_packet_buffer = vec![0u8; app::MAX_PACKET_SIZE];

        loop {
            let (_packet, client) = router.recv_from(&mut big_packet_buffer).expect("The server router failed to receive? Dead?");

            if let Ok(agent) = Agent::start(None, &client.ip().to_string(), client.port()) {
                let hand_shake = HandShake::new(agent.port());
                //send this client a packet back with the port of the connection
                router.send_to(app::serialize(&hand_shake).unwrap().as_slice(), client).expect("Failed to send hand_shake?");

                //create a connection and spawn a thread to manage it
                let mut conns = connections.lock().unwrap();
                conns.push((client, agent))
            }
        }
    }
}

