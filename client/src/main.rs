use client::open_connection;
use serde::{Serialize, Deserialize};


fn main() {
    open_connection("127.0.0.1").unwrap(); 
}
