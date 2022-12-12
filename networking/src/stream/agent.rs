use std::{error::Error, thread::JoinHandle, sync::mpsc, net::SocketAddr};
use serde::{Serialize, de::DeserializeOwned};
use crate::{ipv4_from_str, serialization::{deserialize, serialize}};
use super::Connection;

//think about changing agent to support encoding then sending
pub struct Agent<S, R> {
    local_addr: SocketAddr,
    handle: Option<JoinHandle<Result<(), std::io::Error>>>,

    receiver: mpsc::Receiver<R>,
    sender: mpsc::Sender<S>,

    signal: mpsc::Receiver<()>,
}

impl<S, R> Drop for Agent<S, R> {
    fn drop(&mut self) {
        match self.handle.take().expect("Agent already joined????").join() {
            Ok(Err(ioe)) => {
                eprintln!("Io error: {:?}", ioe);
            },
            Err(te) => {
                eprintln!("An agent thread panicked: {:?}", te);
            },
            _ => {
                println!("Lost connection!");
            }
        }
    }
}

//TODO think about using tokio to manage these tasks
impl<S, R> Agent<S, R> 
where
    S: Send + Sync + std::fmt::Debug + Serialize + 'static,
    R: Send + Sync + std::fmt::Debug + DeserializeOwned + 'static,
{
    pub fn local_addr(&self) -> SocketAddr { self.local_addr }

    pub fn start(host_port: Option<u16>, ip: &str, client_port: u16) -> Result<Self, Box<dyn Error>> {

        let client_addr = SocketAddr::new(ipv4_from_str(ip)?, client_port);
        let (connection, port) = Connection::new(client_addr, host_port)?;
        let local_addr = SocketAddr::new(ipv4_from_str("127.0.0.1")?, port);

        let (thread_sender, receiver) = mpsc::channel::<R>();
        let (sender, thread_receiver) = mpsc::channel::<S>();

        let (signaller, signal) = mpsc::channel::<()>();
        
        let handle = Some(std::thread::spawn(move || { 
            let result = Self::handle_connection(connection, thread_sender, thread_receiver); 
            signaller.send(()).expect("Thread connection was dropped before being joined!");
            result
        }));

        
        Ok(Self {
            local_addr,
            handle,
            receiver,
            sender,
            signal,
        })
    }

    pub fn lost_connection(&self) -> bool {
        if let Ok(_) = self.signal.try_recv() {
            true 
        } else {
            false
        }
    }

    //change this to know how when close
    pub fn get_messages(&self) -> Vec<R> {
        let mut messages = Vec::new();
        while let Ok(message) = self.receiver.try_recv() {
            messages.push(message);
        }
        messages
    }

    pub fn send_message(&self, messages: S) {
        let _result = self.sender.send(messages);
    }

    fn handle_connection(mut connection: Connection, sender: mpsc::Sender<R>, receiver: mpsc::Receiver<S>) -> Result<(), std::io::Error> {
        loop {
            let message = connection.receive_bytes()?; 
            if let Some(message) = message {
                //decode the message here
                let message = match deserialize::<R>(&message) {
                    Some(sm) => sm,
                    None => continue,
                };
                sender.send(message).expect("Thread connection was dropped before being joined");

                //ack the packet
                //connection.send_bytes(&[])?;
            }

            //try and read a message and send it
            if let Ok(packet) = receiver.try_recv() {
                connection.send_bytes(&serialize(&packet).unwrap())?;
            }
        }
    }
}
