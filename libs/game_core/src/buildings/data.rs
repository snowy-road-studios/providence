use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Resource, Debug, Serialize, Deserialize, Clone)]
pub struct BuildingData
{
    pub hq: HqLevels,
}
