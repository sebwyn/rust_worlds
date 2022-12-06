#[macro_use]
extern crate bitflags;

mod core;
mod graphics;
mod app;

mod scenes;
use scenes::Voxels;

pub use app::App;
