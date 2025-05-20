use std::borrow::{Borrow, Cow};

use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use serde::Deserialize;
use utils::RootConfigs;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Component with the canonical ID of a type of tile.
#[derive(Component, Debug, Clone, Deserialize, Eq, PartialEq, Hash, Reflect)]
#[reflect(Hash)]
#[component(immutable)]
pub struct TileId(Cow<'static, str>);

impl TileId
{
    pub fn new(id: impl AsRef<str>) -> Self
    {
        Self(Cow::from(String::from(id.as_ref())))
    }

    pub fn get(&self) -> &str
    {
        &self.0
    }
}

impl Borrow<str> for TileId
{
    fn borrow(&self) -> &str
    {
        self.get()
    }
}

//-------------------------------------------------------------------------------------------------------------------

// TODO: add config for 'minimum distance to nearest tile of same tile'? may be hard to enforce this efficiently
// - may be useful to improve competitive fairness regarding hyperium locations
#[derive(Debug, Deserialize, Clone)]
pub struct TileSpec
{
    pub is_ownable: bool,
    /// Used to determine the frequency this tile is generated. Proportional to sum of all tile mapgen factors.
    pub mapgen_factor: u64,
    pub builds_into: Vec<BuildingId>,
    /// Special flag for marking water tiles which can have proximity effects.
    #[serde(default)]
    pub is_water_tile: bool,
    #[serde(default)]
    pub is_edge_tile: bool,
}

impl TileSpec
{
    fn validate(&self, id: &TileId, buildings: &BuildingData) -> Result<(), String>
    {
        if !self.is_ownable && !self.builds_into.is_empty() {
            return Err(format!("{:?} has spec with !is_ownable but its builds_into isn't empty", id));
        }

        let mut building_ids = HashSet::with_capacity(self.builds_into.len());
        for building_id in self.builds_into.iter() {
            if !building_ids.insert(building_id.clone()) {
                return Err(format!("{:?} has spec with duplicate builds_into entry {:?}", id, building_id));
            }
            if !buildings.contains_key(building_id) {
                return Err(format!("{:?} has spec with unregistered builds_into entry {:?}", id, building_id));
            }
        }

        if self.mapgen_factor == 0 {
            return Err(format!("{:?} has spec with a mapgen factor of 0", id));
        }

        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Debug, Deserialize, Clone, Deref)]
pub struct TileData(HashMap<TileId, TileSpec>);

impl TileData
{
    pub fn new(configs: &RootConfigs) -> Result<Self, String>
    {
        configs.get_type_from_file::<Self>("tile_data")
    }

    pub(crate) fn validate(&self, buildings: &BuildingData) -> Result<(), String>
    {
        let mut edge_count = 0;
        for (id, spec) in self.iter() {
            if spec.is_edge_tile {
                edge_count += 1;
            }

            spec.validate(id, buildings)?;
        }

        if edge_count != 1 {
            return Err(
                format!("tile data does not have exactly one tile spec marked as an edge tile; \
                edge tiles = {edge_count}"),
            );
        }

        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------------------------
