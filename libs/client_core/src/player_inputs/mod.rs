#[cfg(feature = "dev")]
mod handle_dev_inputs;
mod handle_player_inputs;
mod plugin;

#[cfg(feature = "dev")]
pub(crate) use handle_dev_inputs::*;
pub(crate) use handle_player_inputs::*;
pub use plugin::*;
