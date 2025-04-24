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

/// Special developer inputs that can be sent to the game.
#[cfg(feature = "commands")]
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum CommandInput
{
    EndGame,
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
    #[cfg(feature = "commands")]
    CommandInput(CommandInput),
}

impl IntoChannel for ClientRequest
{
    fn into_event_type(&self) -> Channel
    {
        match &self {
            Self::GetGameState => SendOrdered.into(),
            Self::PlayerInput(input) => input.into_event_type(),
            #[cfg(feature = "commands")]
            Self::CommandInput(_) => SendOrdered.into(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
