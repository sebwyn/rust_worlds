#[macro_use]
extern crate bitflags;

mod core;
mod graphics;
mod app;

//mod rotating_tri;
//use rotating_tri::RotatingTri;

//mod cpu_voxels;
//use cpu_voxels::CpuVoxels;

mod voxels;
use voxels::Voxels;

mod vertex;
pub use vertex::Vert;

pub use app::App;
