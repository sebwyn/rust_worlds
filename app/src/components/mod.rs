mod player;

#[cfg(feature = "server")]
pub use player::Player;


#[cfg(feature = "client")]
pub use player::CPlayer;
