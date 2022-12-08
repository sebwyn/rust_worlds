//this is the standalone server app

use std::error::Error;

use server::Server;

fn main() -> Result<(), Box<dyn Error>> {
    Server::default().run()
}
