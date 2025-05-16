use bevy::prelude::*;
use serde::Deserialize;
use utils::RootConfigs;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct GameData
{
    pub mapgen_settings: MapGenSettings,
    pub resources: ResourceData,
    pub tiles: TileData,
    pub buildings: BuildingData,
}

impl GameData
{
    pub fn new(configs: &RootConfigs) -> Result<Self, String>
    {
        let data = Self {
            mapgen_settings: configs.get_type::<MapGenSettings>("game", "MAPGEN_SETTINGS")?,
            resources: ResourceData::new(),
            tiles: TileData::new(configs)?,
            buildings: BuildingData::new(configs)?,
        };

        data.validate()?;

        Ok(data)
    }

    /// Destructures the game data into resources.
    pub fn insert(self, world: &mut World)
    {
        world.insert_resource(self.mapgen_settings);
        world.insert_resource(self.tiles);
        world.insert_resource(self.buildings);
    }

    fn validate(&self) -> Result<(), String>
    {
        self.resources.validate()?;
        self.buildings.validate(&self.resources)?;
        self.tiles.validate(&self.buildings)?;
        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------------------------
