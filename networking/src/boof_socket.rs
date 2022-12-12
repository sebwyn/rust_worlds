use std::{net::{UdpSocket, SocketAddr, ToSocketAddrs}, time::Duration};

use random::Source;

pub struct BoofSocket {
    socket: UdpSocket,

    packet_loss_freq: f64, //as a percentage
    latency: u32, //in ms
    rand: random::Default,
}

impl std::fmt::Debug for BoofSocket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoofSocket").field("socket", &self.socket).field("packet_loss_freq", &self.packet_loss_freq).field("latency", &self.latency).finish()
    }
}

impl BoofSocket {
    pub fn set_latency(&mut self, ms: u32) {
        self.latency = ms;
    }

    pub fn set_packet_loss_freq(&mut self, percent: f64){
        self.packet_loss_freq = percent;
    }

    pub fn socket(&self) -> &UdpSocket { &self.socket }
}

impl BoofSocket {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> std::io::Result<BoofSocket> {
        Ok(Self {
            socket: UdpSocket::bind(addr)?,
            
            packet_loss_freq: 0.0,
            latency: 0,
            rand: random::default(47013984),
        })
    }

    pub fn set_read_timeout(&self, dur: Option<Duration>) -> std::io::Result<()> {
        self.socket.set_read_timeout(dur)
    }

    pub fn recv_from(&self, buf: &mut [u8]) -> std::io::Result<(usize, SocketAddr)> {
        self.socket.recv_from(buf)
    }

    //for now just do everything in send and set in on the client, this
    //is pretty innacurate to real life where im pretty sure packets could be dropped 
    //from either end
    //
    //this could probably be done by just setting latency on one end
    //setting packet loss on both ends to be the same
    pub fn send_to<A: ToSocketAddrs>(&mut self, buf: &[u8], addr: A) -> std::io::Result<usize> {
        //wait the simulate latency before sending
        std::thread::sleep(Duration::from_millis(self.latency as u64));

        //simulate packet loss by randomly not sending a packet
        if self.rand.read_f64() > self.packet_loss_freq {
            self.socket.send_to(buf, addr)
        } else {
            Ok(buf.len())
        }
    }
}
