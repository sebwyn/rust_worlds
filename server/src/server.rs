use networking::stream::Agent;
use std::{
    collections::HashMap,
    error::Error,
    net::{SocketAddr, UdpSocket},
    sync::{Arc, Mutex},
    time::Duration,
};

use app::components::Player;

use app::{ClientEvent, HandShake, Snapshot};

type Receiving = Vec<ClientEvent>;
type Sending = Snapshot;

//think about tokio for dealing with all these threads
pub struct Server {
    connections: Arc<Mutex<Vec<(SocketAddr, Agent<Sending, Receiving>)>>>,

    next_player_id: u32,
    player_ids: HashMap<SocketAddr, u32>,
    //game state
    players: HashMap<u32, Player>,

    tick_rate: u64,
}

impl Server {
    pub fn new(tick_rate: u64) -> Self {
        Self {
            connections: Arc::new(Mutex::new(Vec::new())),
            tick_rate,
            players: HashMap::new(),
            player_ids: HashMap::new(),
            next_player_id: 0,
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

            //update players based on received events
            for (client, events) in client_messages {
                let player_id = self.player_ids.entry(client).or_insert_with(|| {
                    self.next_player_id += 1;
                    self.next_player_id
                });
                let player = self.players.entry(*player_id).or_insert(Player::new());
                player.update(events);
            }

            let player_transforms: HashMap<u32, app::Transform> = self
                .players
                .iter()
                .map(|(k, player)| (*k, player.transform()))
                .collect();

            if self.players.len() > 0 {
                let clients = self.connections.lock().unwrap();
                for (client, agent) in clients.iter() {
                    let id = match self.player_ids.get(client) {
                        Some(id) => id,
                        None => break,
                    };

                    let snapshot = app::Snapshot {
                        player_transforms: player_transforms.clone(),
                        local_id: *id,
                    };

                    println!("{}: {}", id, snapshot.player_transforms.len());
                    //send a vector of messages excluding messages from this client
                    agent.send_message(snapshot);
                }
            }

            std::thread::sleep(Duration::from_millis(wait_time));
        }
    }

    fn close_stale_connections(&mut self) {
        let mut clients = self.connections.lock().unwrap();

        clients.retain(|(addr, agent)| {
            if agent.lost_connection() {
                let id = self.player_ids.remove(addr).unwrap();
                self.players.remove(&id);
                false
            } else {
                true
            }
        });
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

    fn listen_thread(connections: Arc<Mutex<Vec<(SocketAddr, Agent<Sending, Receiving>)>>>) {
        let router = UdpSocket::bind("0.0.0.0:6669").expect("Failed to open listen thread");

        //I'm assuming each send gets mapped to one recv, but idk, who knows, we'll find out
        let mut big_packet_buffer = vec![0u8; app::MAX_PACKET_SIZE];

        loop {
            let (_packet, client) = router
                .recv_from(&mut big_packet_buffer)
                .expect("The server router failed to receive? Dead?");

            let client_ip = client.ip().to_string();

            if let Ok(agent) = Agent::start(None, &client_ip, client.port()) {
                let hand_shake = HandShake {
                    port: agent.local_addr().port(),
                };
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
