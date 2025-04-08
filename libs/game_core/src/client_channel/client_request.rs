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
#[cfg(feature = "dev")]
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum DevInput
{
    EndGame,
}

//-------------------------------------------------------------------------------------------------------------------

/// Requests that can be sent to the game.
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
pub enum ClientRequest
{
    /// Request the current game mode.
    GetGameState,
    /// Player input.
    PlayerInput(PlayerInput),
    #[cfg(feature = "dev")]
    DevInput(DevInput),
}

impl IntoChannel for ClientRequest
{
    fn into_event_type(&self) -> Channel
    {
        match &self {
            Self::GetGameState => SendOrdered.into(),
            Self::PlayerInput(input) => input.into_event_type(),
            #[cfg(feature = "dev")]
            Self::DevInput(input) => SendOrdered.into(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
