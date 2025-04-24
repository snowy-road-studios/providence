use bevy_girk_utils::*;
use bevy_replicon::prelude::Channel;
use serde::{Deserialize, Serialize};

//-------------------------------------------------------------------------------------------------------------------

/// Player inputs that can be sent to the game.
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum PlayerInput
{
    Placeholder,
}

impl IntoChannel for PlayerInput
{
    fn into_event_type(&self) -> Channel
    {
        match &self {
            Self::Placeholder => SendUnordered.into(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Special commands that can be sent to the game.
///
/// Commands do nothing unless the `"commands"` feature is enabled.
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum CommandInput
{
    NextRound,
    Pause,
    Unpause,
    EndGame,
}

impl CommandInput
{
    /// Returns true if the server is able to process commands.
    ///
    /// Can be used to validate that commands are disabled in prod.
    pub const fn command_processing_enabled() -> bool
    {
        #[cfg(feature = "commands")]
        return true;

        #[cfg(not(feature = "commands"))]
        return false;
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Requests that can be sent to the game.
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum ClientRequest
{
    /// Request the current game state.
    GetGameState,
    /// Player input.
    PlayerInput(PlayerInput),
    CommandInput(CommandInput),
}

impl IntoChannel for ClientRequest
{
    fn into_event_type(&self) -> Channel
    {
        match &self {
            Self::GetGameState => SendOrdered.into(),
            Self::PlayerInput(input) => input.into_event_type(),
            Self::CommandInput(_) => SendOrdered.into(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
