use networking::stream::Agent;
use std::{
    error::Error,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
    time::Duration, collections::HashMap,
};

use app::components::Player;

use app::{ClientEvent, HandShake, Snapshot};

type Receiving = Vec<ClientEvent>;
type Sending = Snapshot;

//think about tokio for dealing with all these threads
pub struct Server {
    connections: Arc<Mutex<Vec<(SocketAddr, Agent<Sending, Receiving>)>>>,

    players: HashMap<SocketAddr, Player>,

    tick_rate: u64,
}

impl Server {
    pub fn new(tick_rate: u64) -> Self {
        Self {
            connections: Arc::new(Mutex::new(Vec::new())),
            tick_rate,
            players: HashMap::new()
        }
    }

    //think about using tcp for routing, and handshaking
    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        {
            let connections = self.connections.clone();
            std::thread::spawn(|| Self::listen_thread(connections));
        }

        let wait_time = 1000u64 / self.tick_rate; //in milliseconds

        loop {
            self.close_stale_connections();
            let client_messages: Vec<(SocketAddr, Receiving)> = self.get_client_messages();

            let mut snapshot = Snapshot(Vec::new());
            //update our world (modify our messages, so they look better)
            for (client, events) in client_messages {
                let player = self.players.entry(client).or_insert(Player::new());
                player.update(events);
                snapshot.0.push(app::GameObject::Player { addr: client, transform: player.transform() });
            }

            self.send_clients_messages(snapshot);

            std::thread::sleep(Duration::from_millis(wait_time));
        }

        //Ok(())
    }

    fn close_stale_connections(&mut self) {
        let mut clients = self.connections.lock().unwrap();

        clients.retain(|(_addr, agent)| !agent.lost_connection());
    }

    //loop and update our clients based on input
    fn get_client_messages(&mut self) -> Vec<(SocketAddr, Receiving)> {
        let clients = self.connections.lock().unwrap();

        clients
            .iter()
            .map(|client| {
                (
                    client.0,
                    client.1.get_messages().into_iter().flatten().collect(),
                )
            })
            .collect()
    }

    //send a message to our clients with the updated world state
    fn send_clients_messages(&self, message: Sending) {
        let clients = self.connections.lock().unwrap();

        for (_, agent) in clients.iter() {
            //send a vector of messages excluding messages from this client
            agent.send_message(message.clone());
        }
    }

    fn listen_thread(connections: Arc<Mutex<Vec<(SocketAddr, Agent<Sending, Receiving>)>>>) {
        let router = UdpSocket::bind("127.0.0.1:6669").expect("Failed to open listen thread");

        //I'm assuming each send gets mapped to one recv, but idk, who knows, we'll find out
        let mut big_packet_buffer = vec![0u8; app::MAX_PACKET_SIZE];

        loop {
            let (_packet, client) = router
                .recv_from(&mut big_packet_buffer)
                .expect("The server router failed to receive? Dead?");

            if let Ok(agent) = Agent::start(None, &client.ip().to_string(), client.port()) {
                let hand_shake = HandShake::new(agent.local_addr().port());
                //send this client a packet back with the port of the connection
                router
                    .send_to(app::serialize(&hand_shake).unwrap().as_slice(), client)
                    .expect("Failed to send hand_shake?");

                //create a connection and spawn a thread to manage it
                let mut conns = connections.lock().unwrap();
                conns.push((client, agent))
            }
        }
    }
}
