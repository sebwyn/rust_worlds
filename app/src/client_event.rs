use serde::{Serialize, Deserialize};
use winit::event::{VirtualKeyCode, MouseButton};

//the server will be responsible for running player controllers
//this means that it will generate appropriate matrices and be able to perform raycasts
//in order to generate matrices, we need to be aware of the client resolution

#[derive(Serialize, Deserialize, Clone)]
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
