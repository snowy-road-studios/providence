use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_attributes::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Player id component (wraps the player's client id).
#[derive(Component, Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct PlayerId
{
    pub id: ClientId,
}

//-------------------------------------------------------------------------------------------------------------------

/// Player name component.
#[derive(Component, Default, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub struct PlayerName
{
    pub name: String,
}

//-------------------------------------------------------------------------------------------------------------------

/// Players are entities with the components bundled here.
#[derive(Bundle)]
pub struct PlayerState
{
    /// Player id. Can be used to access player context from game context.
    pub id: PlayerId,
    /// Player name.
    pub name: PlayerName,
    /// Players are replicated
    pub replicate: Replicated,
    /// Players have a visibility condition.
    pub visibility: VisibilityCondition,
}

impl Default for PlayerState
{
    fn default() -> Self
    {
        Self {
            id: PlayerId { id: 1 },
            name: Default::default(),
            replicate: Default::default(),
            visibility: vis![Global],
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
