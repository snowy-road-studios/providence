use bevy_girk_utils::*;
use bevy_replicon::prelude::Channel;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Reasons a game request may be rejected
#[derive(Debug, Serialize, Deserialize)]
pub enum RejectionReason
{
    ModeMismatch,
    Invalid,
    None,
}

//-------------------------------------------------------------------------------------------------------------------

/// Messages that can be sent out of the game.
#[derive(Debug, Serialize, Deserialize)]
pub enum GameMsg
{
    RequestRejected
    {
        reason: RejectionReason,
        request: ClientRequest,
    },
    CurrentGameState(GameState),
    TileSelectInfo
    {
        remaining_ms: u128,
    },
    RoundInfo
    {
        round: u32,
        remaining_ms: u128,
    },
    Pause,
    Unpause,
}

impl IntoChannel for GameMsg
{
    fn into_event_type(&self) -> Channel
    {
        match &self {
            Self::RequestRejected { .. } => SendUnordered.into(),
            Self::CurrentGameState(_) => SendOrdered.into(),
            Self::TileSelectInfo { .. } => SendOrdered.into(),
            Self::RoundInfo { .. } => SendOrdered.into(),
            Self::Pause => SendOrdered.into(),
            Self::Unpause => SendOrdered.into(),
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
