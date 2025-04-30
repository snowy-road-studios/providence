use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use utils::RootConfigs;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameData
{
    pub buildings: BuildingData,
}

impl GameData
{
    pub fn new(configs: &RootConfigs) -> Result<Self, String>
    {
        let hq_levels =
            HqLevels::deserialize(configs.get_value("hq", "LEVELS")?.clone()).map_err(|err| format!("{err:?}"))?;
        let buildings = BuildingData { hq: hq_levels };

        Ok(Self { buildings })
    }

    /// Destructures the game data into resources.
    pub fn insert(self, world: &mut World)
    {
        world.insert_resource(self.buildings);
    }
}

//-------------------------------------------------------------------------------------------------------------------
