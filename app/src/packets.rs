use std::net::SocketAddr;

use serde::{Serialize, Deserialize};
use winit::event::{VirtualKeyCode, MouseButton};

//the server will be responsible for running player controllers
//this means that it will generate appropriate matrices and be able to perform raycasts
//in order to generate matrices, we need to be aware of the client resolution

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ClientEvent {
    //input events
    KeyPressed(VirtualKeyCode),
    KeyReleased(VirtualKeyCode),
    MousePressed((MouseButton, (f64, f64))),
    MouseReleased((MouseButton, (f64, f64))),
    CursorMoved((f64, f64)),
    
    //resolution changes, used to decode mouse events
    ScreenSizeChanged((f64, f64)),
} 

//using the server to derive the local ip, kind strange
#[derive(Serialize, Deserialize)]
pub struct HandShake {
    pub client_ip: String,
    pub port: u16,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 2]
}


#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum GameObject {
    Player { 
        addr: SocketAddr,
        transform: Transform,
    },
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Snapshot(pub Vec<GameObject>);
