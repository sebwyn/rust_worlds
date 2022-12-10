use std::{error::Error, thread::JoinHandle, sync::mpsc, net::SocketAddr};
use serde::{Deserialize, Serialize};
use crate::{Connection, ipv4_from_str, serialization::{deserialize, serialize}};

pub enum AgentError { 
    ThreadPanicked,
    Io(std::io::Error),
}


pub struct Agent<S, R> {
    port: u16,
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
    R: Send + Sync + std::fmt::Debug + for<'de> Deserialize<'de> + 'static,
{
    pub fn port(&self) -> u16 { self.port }

    pub fn start(host_port: Option<u16>, ip: &str, client_port: u16) -> Result<Self, Box<dyn Error>> {

        let client_addr = SocketAddr::new(ipv4_from_str(ip)?, client_port);
        let (connection, port) = Connection::new(client_addr, host_port)?;

        let (thread_sender, receiver) = mpsc::channel::<R>();
        let (sender, thread_receiver) = mpsc::channel::<S>();

        let (signaller, signal) = mpsc::channel::<()>();
        
        let handle = Some(std::thread::spawn(move || { 
            let result = Self::handle_connection(connection, thread_sender, thread_receiver); 
            signaller.send(()).expect("Thread connection was dropped before being joined!");
            result
        }));

        
        Ok(Self {
            port,
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
                connection.send_bytes(&[])?;
            }

            //try and read a message and send it
            if let Ok(packet) = receiver.try_recv() {
                connection.send_bytes(&serialize(&packet).unwrap())?;
            }
        }
    }
}
