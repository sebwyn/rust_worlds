#[macro_use]
extern crate bitflags;

mod core;
mod graphics;

mod scenes;
use scenes::Voxels;

mod worlds;
pub use worlds::App;
