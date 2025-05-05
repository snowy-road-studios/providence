use bevy::prelude::*;
use serde::Deserialize;
use utils::RootConfigs;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct GameData
{
    pub mapgen_settings: MapGenSettings,
    pub buildings: BuildingData,
}

impl GameData
{
    pub fn new(configs: &RootConfigs) -> Result<Self, String>
    {
        let mapgen_settings = configs.get_type::<MapGenSettings>("game", "MAPGEN_SETTINGS")?;

        let hq_levels = configs.get_type::<HqLevels>("hq_data", "LEVELS")?;
        let buildings = BuildingData { hq: hq_levels };

        Ok(Self { mapgen_settings, buildings })
    }

    /// Destructures the game data into resources.
    pub fn insert(self, world: &mut World)
    {
        world.insert_resource(self.mapgen_settings);
        world.insert_resource(self.buildings);
    }
}

//-------------------------------------------------------------------------------------------------------------------
