use crate::{
    serialization::{deserialize, serialize, serialized_size},
    BoofSocket,
};

use serde::{Deserialize, Serialize};
use std::{
    error::Error,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    time::Duration,
};

#[derive(Serialize, Deserialize, Default, Debug, Copy, Clone)]
pub struct Header {
    seq: u16,
    ack: u16,
    acks: u32,
}

const BUFFER_SIZE: usize = 1024;

//TODO: handle really old packets, that may break the connection
//a connection merely implements the raw reliable udp interface, packet definitions are up to you!
#[derive(Debug)]
pub struct Connection {
    sequence: u16,
    ack: u16,
    acks: u32,

    //packet and whether its been acked
    sent_headers: Vec<(Header, bool)>, //[(Packet<PayLoad>, bool); BUFFER_SIZE],
    received_headers: Vec<(Header, bool)>, //[(Packet<PayLoad>, bool); BUFFER_SIZE],

    local_address: SocketAddr,
    client_address: SocketAddr,
    socket: BoofSocket,
}

impl Connection {
    pub fn client_addr(&self) -> &SocketAddr {
        &self.client_address
    }
    pub fn local_addr(&self) -> &SocketAddr {
        &self.local_address
    }
}

impl Connection {
    //can specify zero here if the port is irrelevant
    pub fn new(
        host_port: Option<u16>,
        client_address: &str,
        client_port: u16,
    ) -> Result<Self, Box<dyn Error>> {
        //the actual socket underlying the connection
        let socket = BoofSocket::bind(
            ("0.0.0.0", host_port.unwrap_or(0))
        )?;

        //connection details
        let host_address = socket.socket().local_addr()?;
        let client_address = SocketAddr::new(
            IpAddr::V4(Ipv4Addr::from_str(client_address)?),
            client_port,
        );
        println!("client port: {}", client_port);

        //socket options for streaming
        socket.socket().set_nonblocking(true)?;
        socket.set_read_timeout(Some(Duration::from_nanos(10)))?;

        //protocol implementation
        let sent_headers = vec![(Header::default(), false); BUFFER_SIZE];
        let received_headers = vec![(Header::default(), false); BUFFER_SIZE];

        Ok(Self {
            sequence: 0,
            ack: 0,
            acks: 0,

            sent_headers,
            received_headers,

            local_address: host_address,
            client_address,
            socket,
        })
    }

    pub fn fake_packet_loss(&mut self, percent: f64) {
        self.socket.set_packet_loss_freq(percent);
    }

    pub fn fake_latency(&mut self, ms: u32) {
        self.socket.set_latency(ms);
    }

    pub fn send_bytes(&mut self, bytes: &[u8]) -> Result<(), std::io::Error> {
        let header = Header {
            seq: self.sequence,
            ack: self.ack,
            acks: self.acks,
        };
        
        let mut packet_bytes = serialize(&header).unwrap();
        packet_bytes.extend_from_slice(bytes);

        // let header_len = packet_bytes.len();
        // packet_bytes.resize(packet_bytes.len() + bytes.len(), 0u8);
        // packet_bytes[header_len..].copy_from_slice(bytes);

        self.socket.send_to(&packet_bytes, self.client_address)?;

        self.sequence = self.sequence.wrapping_add(1);
        Ok(())
    }

    //defaults to a none blocking receive
    pub fn receive_bytes(&mut self) -> Result<Option<Vec<u8>>, std::io::Error> {
        //receive all the packets waiting for us, updating our connection
        let mut max_packet_buffer = [0u8; super::MAX_PACKET_SIZE];
        let (_bytes, from) = match self.socket.recv_from(&mut max_packet_buffer) {
            Ok(received) => received,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => return Ok(None),
            Err(e) => return Err(e),
        };

        assert!(
            from == self.client_address,
            "Connection is trying to receive from the wrong client!"
        );

        //read in our header
        let header = deserialize::<Header>(&max_packet_buffer).unwrap();
        let header_size = serialized_size(&header).unwrap() as usize;

        //get an index into our received packets at this packets location
        let index = header.seq as usize % BUFFER_SIZE;

        //think about wrapping the two iterations here into 1 loop, however, this is less readable

        //mark packets acknowledged
        //s should align with the sequence number on our end that needs to be acked
        for b in (0u16..32u16).rev() {
            let s = header.ack.wrapping_sub(b);
            //get header data here
            if (header.acks & 1 << b) > 0 {
                let index = s as usize % BUFFER_SIZE;
                if self.sent_headers[index].0.seq == s {
                    self.sent_headers[index].1 = true;
                }
            }
        }

        //add this packet to our received packets
        if header.seq > self.ack
            || (header.seq < BUFFER_SIZE as u16 && self.ack > u16::MAX - BUFFER_SIZE as u16)
        {
            self.ack = header.seq;
        }
        //this line is potentially problematic, if we're receiving a really old packet see TODO
        self.received_headers[index] = (header, true);

        //no chance we actually have to rebuild this everytime we receive a packet
        //but this is a good template
        self.acks = 0u32;
        for b in 0u16..32u16 {
            let ack = self.ack.wrapping_sub(b);
            //get packet from received packets here
            let index = ack as usize % BUFFER_SIZE;
            if self.received_headers[index].0.seq == ack && self.received_headers[index].1 {
                //set the bit in our acks field
                self.acks |= 1 << b;
            }
        }

        //read in the rest of the message as the payload
        let mut payload = vec![0u8; super::MAX_PACKET_SIZE - header_size];
        payload[..].copy_from_slice(&max_packet_buffer[header_size..]);
        
        Ok(Some(payload))
    }
}
