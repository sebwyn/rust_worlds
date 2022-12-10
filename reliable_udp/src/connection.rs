use crate::BoofSocket;

use std::{net::{IpAddr, SocketAddr, Ipv4Addr}, error::Error, time::Duration};

use regex::Regex;

//think about using serde for serializing and deserializing??
#[derive(Default, Debug, Copy, Clone)]
struct Packet<PayLoad> {
    seq: u16,
    ack: u16,
    acks: u32,
    //payload needs to be a type that can be converted to bytes and decoded
    payload: PayLoad
}

const BUFFER_SIZE: usize = 1024;


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
//
//a connection merely implements the raw reliable udp interface, packet definitions are up to you!
#[derive(Debug)]
pub struct Connection<PayLoad> {
    sequence: u16,
    ack: u16,
    acks: u32,


    //packet and whether its been acked
    sent_packets: Vec<(Packet<PayLoad>, bool)>,//[(Packet<PayLoad>, bool); BUFFER_SIZE],
    received_packets: Vec<(Packet<PayLoad>, bool)>,//[(Packet<PayLoad>, bool); BUFFER_SIZE],

    _host_address: SocketAddr,
    client_address: SocketAddr,
    socket: BoofSocket,
}

impl<P> Connection<P> {
    pub fn client_address(&self) -> SocketAddr { self.client_address }
}

impl<PayLoad> Connection<PayLoad> 
where
    PayLoad: std::default::Default + std::marker::Copy + std::fmt::Debug
{
    //can specify zero here if the port is irrelevant
    pub fn new(client_address: SocketAddr, port: Option<u16>) -> Result<(Self, u16), Box<dyn Error>> 
    {
        //make our connection here    
        let host_address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port.unwrap_or(0));
        let socket = BoofSocket::bind(host_address)?;

        //socket.set_nonblocking(true)?;
        socket.set_read_timeout(Some(Duration::from_nanos(10)))?;

        let port = socket.socket().local_addr()?.port();

        let mut sent_packets = Vec::new();
        sent_packets.resize(BUFFER_SIZE, (Packet::default(), false));
        let mut received_packets = Vec::new();
        received_packets.resize(BUFFER_SIZE, (Packet::default(), false));

        Ok((Self {
            sequence: 0,
            ack: 0,
            acks: 0,

            _host_address: host_address,
            client_address,

            sent_packets, 
            received_packets,

            socket,
        }, port))
    }

    pub fn fake_packet_loss(&mut self, percent: f64){
        self.socket.set_packet_loss_freq(percent);
    }

    pub fn fake_latency(&mut self, ms: u32){
        self.socket.set_latency(ms);
    }

    pub fn send_packet(&mut self, payload: PayLoad) -> Result<(), Box<dyn Error>> {

        let mut packet = Packet { seq: self.sequence, ack: self.ack, acks: self.acks, payload };
        let packet_bytes = &*unsafe { to_bytes(&mut packet) };
        self.socket.send_to(packet_bytes, self.client_address)?;
        println!("Sending packet: ack: {} acks {1:#032b}", self.ack, self.acks);
        
        self.sequence += 1;

        Ok(())
    }

    //defaults to a none blocking receive
    pub fn receive_packet(&mut self) -> Result<Option<PayLoad>, Box<dyn Error>> {
        //receive all the packets waiting for us, updating our connection
        let mut packet = Packet::default();
        let packet_bytes = unsafe { to_bytes(&mut packet) };

        let (_bytes, from) = match self.socket.recv_from(packet_bytes) {
            Ok(received) => received,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                return Ok(None)
            },
            Err(e) => return Err(Box::new(e))
        };
        assert!(from == self.client_address, "Connection is trying to receive from the wrong client!");
        
        //get an index into our received packets at this packets location
        let index = packet.seq as usize % BUFFER_SIZE;

        //catch a really old packet
        if self.received_packets[index].0.seq > packet.seq {
            return Ok(None);
        }

        //mark packets acknowledged
        //s should align with the sequence number on our end that needs to be acked
        for b in (0u32..32u32).rev() {
            let s = packet.ack as i32 - b as i32;
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

        //add this packet to our received packets
        if packet.seq >= self.ack {
            self.ack = packet.seq;
            //ack our packet here
            self.received_packets[index] = (packet, true);
        } else {
            //so the issue here is we could possibly receive a really old packet that would
            //overwrite some of our good data
            self.received_packets[index] = (packet, true);
        }
        
        //no chance we actually have to rebuild this everytime we receive a packet
        //but this is a good template
        self.acks = 0u32;
        for b in 0u16..32u16 {
            let ack = self.ack as i32 - b as i32;

            let ack: u16 = match ack.try_into() {
                Ok(i) => i,
                Err(_) => break
            };

            //get packet from received packets here
            let index = ack as usize % BUFFER_SIZE;
            if self.received_packets[index].0.seq == ack && self.received_packets[index].1 {
                //set the bit in our acks field
                self.acks |= 1 << b; 
            }
        }

        Ok(Some(packet.payload))
    }
}

unsafe fn to_bytes<T: Sized>(p: &mut T) -> &mut [u8] {
    ::std::slice::from_raw_parts_mut(
        (p as *mut T) as *mut u8,
        ::std::mem::size_of::<T>(),
    )
}
