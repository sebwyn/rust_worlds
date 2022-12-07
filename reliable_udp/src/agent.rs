use std::{net::{IpAddr, UdpSocket, SocketAddr, Ipv4Addr}, error::Error, sync::{Mutex, Arc}, time::Duration};

use regex::Regex;

//think about using serde for serializing and deserializing??
#[derive(Default, Debug, Copy, Clone)]
struct Packet {
    seq: u16,
    ack: u16,
    acks: u32
}

const BUFFER_SIZE: usize = 1024;

pub struct Connection {
    sequence: u16,
    ack: u16,
    acks: u32,

    host_address: SocketAddr,
    client_address: SocketAddr,

    //packet and whether its been acked
    sent_packets: [(Packet, bool); BUFFER_SIZE],
    received_packets: [(Packet, bool); BUFFER_SIZE],

    socket: UdpSocket,
}

#[derive(Debug)]
struct ErrorWithMessage(&'static str);

impl std::fmt::Display for ErrorWithMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}
impl Error for ErrorWithMessage { }

pub fn ipv4_from_str(ip: &str) -> Result<IpAddr, Box<dyn Error>> {
    let regex = Regex::new(r"^(?P<a>\d+)\.(?P<b>\d+)\.(?P<c>\d+)\.(?P<d>\d+)$")?;
    let captures = regex.captures(ip).ok_or(ErrorWithMessage("Invalid ipv4 string"))?;
    let a = captures.name("a").ok_or(ErrorWithMessage("Invalid ipv4 string"))?.as_str().parse::<u8>()?;
    let b = captures.name("b").ok_or(ErrorWithMessage("Invalid ipv4 string"))?.as_str().parse::<u8>()?;
    let c = captures.name("c").ok_or(ErrorWithMessage("Invalid ipv4 string"))?.as_str().parse::<u8>()?;
    let d = captures.name("d").ok_or(ErrorWithMessage("Invalid ipv4 string"))?.as_str().parse::<u8>()?;


    Ok(IpAddr::V4(Ipv4Addr::new(a, b, c, d)))
}

//TODO: remove all these publics
impl Connection {
    //can specify zero here if the port is irrelevant
    pub fn new(host_port: u16, client_ip: IpAddr, client_port: u16) -> Result<Self, Box<dyn Error>> 
    {
        //make our connection here    
        let host_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), host_port);
        let socket = UdpSocket::bind(host_address)?;

        let client_address = SocketAddr::new(client_ip, client_port);

        //socket.set_nonblocking(true)?;
        socket.set_read_timeout(Some(Duration::from_nanos(10)));

        Ok(Self {
            sequence: 0,
            ack: 0,
            acks: 0,

            host_address,
            client_address,

            sent_packets: [(Packet::default(), false); BUFFER_SIZE],
            received_packets: [(Packet::default(), false); BUFFER_SIZE],

            socket,
        })
    }

    pub fn send_packet(&mut self) -> Result<(), Box<dyn Error>> {

        let mut packet = Packet { seq: self.sequence, ack: self.ack, acks: self.acks };
        let packet_bytes = &*unsafe { to_bytes(&mut packet) };
        self.socket.send_to(packet_bytes, self.client_address)?;
        println!("Sending packet: {:?}", packet);
        
        self.sequence += 1;

        Ok(())
    }

    //libc poll??
    /*pub fn poll_packet(&self) -> Result<bool, Box<dyn Error>> {
        let mut packet = Packet::default();
        let packet_bytes = unsafe { to_bytes(&mut packet) };
        let (bytes, _from) = self.socket.peek_from(packet_bytes)?;

        Ok(bytes > 0)
    }*/

    pub fn receive_packets(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut received_packet = false;

        //receive all the packets waiting for us, updating our connection
        loop {
            let mut packet = Packet::default();
            let packet_bytes = unsafe { to_bytes(&mut packet) };

            let (_bytes, from) = match self.socket.recv_from(packet_bytes) {
                Ok(received) => received,
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return Ok(received_packet)
                },
                Err(e) => return Err(Box::new(e))
            };

            println!("Received packet from {:?}: {:?}", from, packet);
            received_packet = true;
            
            //mark packets acknowledged
            //s should align with the sequence number on our end that needs to be acked
            for b in (0u32..32u32).rev() {
                let s = packet.acks as i32 - b as i32;
                if s < 0 { break } //if we're just starting only try to ack packets >= zero
                let s = s as u16;
                
                //get packet data here
                if (packet.acks & 1 >> b) > 0 {
                    let index = s as usize % BUFFER_SIZE;
                    if self.sent_packets[index].0.seq == s {
                        self.sent_packets[index].1 = true;
                    }
                }
            }

            //get an index into our received packets at this packets location
            let index = packet.seq as usize % BUFFER_SIZE;

            //add this packet to our received packets
            if packet.seq > self.ack {
                self.ack = packet.seq;
                //ack our packet here
                self.received_packets[index] = (packet, true);
            } else {
                //if we haven't already received this packet add it
                 
            }
            //potentially slow to do this here in this way, but update our 32 bit ack mask
            for b in (0u32..32u32).rev() {
                let ack = self.ack as i32 - b as i32;
                if ack < 0 { break } //if we're just starting only try to ack packets >= zero
                let ack = ack as u16;
                
                //get packet from received packets here
                let index = ack as usize % BUFFER_SIZE;
                if self.received_packets[index].0.seq == ack && self.received_packets[index].1 {
                    //set the bit in our acks field
                    self.acks |= 1 >> b;
                }
            }
        };
    }
}

pub struct Agent {
    _connection: Arc<Mutex<Connection>>,
}

impl Agent {


}
unsafe fn to_bytes<T: Sized>(p: &mut T) -> &mut [u8] {
    ::std::slice::from_raw_parts_mut(
        (p as *mut T) as *mut u8,
        ::std::mem::size_of::<T>(),
    )
}
