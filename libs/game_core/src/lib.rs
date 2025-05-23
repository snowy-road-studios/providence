mod buildings;
mod client;
mod client_channel;
mod client_connect;
mod game_data;
mod game_rand;
mod map;
mod meta;
mod plugin;
mod prng;
mod resources;
mod rounds;
mod sets;
mod setup;
mod states;
mod time;

/// Re-export
pub(crate) use bevy_girk_game_fw::GameSender;
pub use buildings::*;
pub use client::*;
pub use client_channel::*;
pub(crate) use client_connect::*;
pub use game_data::*;
pub use game_rand::*;
pub use map::*;
pub use meta::*;
pub use plugin::*;
pub use prng::*;
/// Re-export
pub use renet2::ClientId;
pub use resources::*;
pub(crate) use rounds::*;
pub use sets::*;
pub(crate) use setup::*;
pub use states::*;
pub use time::*;
/// Re-export
pub(crate) mod vis
{
    pub(crate) use bevy_replicon_attributes::*;
}
