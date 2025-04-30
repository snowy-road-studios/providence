use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HqLevel
{
    pub ownable_tiles: u16,
    pub cost: ResourceCost,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone, Deref)]
pub struct HqLevels(pub Vec<HqLevel>);

//-------------------------------------------------------------------------------------------------------------------
