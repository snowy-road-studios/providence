use std::borrow::{Borrow, Cow};

use bevy::platform::collections::{HashMap, HashSet};
use bevy::prelude::*;
use serde::Deserialize;
use utils::RootConfigs;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Component with the canonical ID of a type of building.
#[derive(Component, Debug, Deserialize, Clone, Eq, PartialEq, Hash, Reflect)]
#[component(immutable)]
pub struct BuildingId(Cow<'static, str>);

impl BuildingId
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

impl Borrow<str> for BuildingId
{
    fn borrow(&self) -> &str
    {
        self.get()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Cost to construct a building (can be from bare tile or as upgrade from another building).
#[derive(Debug, Deserialize, Clone)]
pub enum BuildCost
{
    System,
    Build
    {
        resources: ResourceCost,
        // TODO: service requirements (optional)
        // TODO: tile proximity requirements (optional)
    },
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub enum BuildingTypeSpec
{
    Hq
    {
        total_tiles_allowed: u16
    },
    Production
    {
        resource_id: ResourceId, production_per_round: u64
    },
    // TODO: Service{
    //     service: ServiceType,  // type of one unit of service
    //     capacity: u64,
    //     // For transport calculations.
    //     capacity_use_weight: u64
    // }
    // TODO: Transportation{
    //     radius: u32,
    //     disrepair_rate: f32
    // }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug, Deserialize, Clone)]
pub struct BuildingSpec
{
    pub build_cost: BuildCost,
    pub builds_into: Vec<BuildingId>,
    pub building_type: BuildingTypeSpec,
    /// Indestructible if None.
    pub destruction_cost: Option<ResourceCost>,
}

impl BuildingSpec
{
    fn validate(&self, id: &BuildingId, buildings: &BuildingData, resources: &ResourceData) -> Result<(), String>
    {
        let mut building_ids = HashSet::with_capacity(self.builds_into.len());
        for building_id in self.builds_into.iter() {
            if !building_ids.insert(building_id.clone()) {
                return Err(format!("{:?} has spec with duplicate builds_into entry {:?}", id, building_id));
            }
            if !buildings.contains_key(building_id) {
                return Err(format!("{:?} has spec with unregistered builds_into entry {:?}", id, building_id));
            }
        }

        match &self.building_type {
            BuildingTypeSpec::Hq { .. } => (),
            BuildingTypeSpec::Production { resource_id, production_per_round } => {
                if !resources.contains(resource_id) {
                    return Err(
                        format!("{:?} has spec for production-type but produced resource {:?} is unregistered",
                        id, resource_id),
                    );
                }
                if *production_per_round == 0 {
                    return Err(format!("{:?} has spec for production-type but production per round is 0", id));
                }
            } /* BuildingTypeSpec::Service { .. } => {
               *     // service unit type is known
               * },
               * BuildingTypeSpec::Transportation { .. } => {
               *     // radius not zero
               * }, */
        }

        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Debug, Deserialize, Clone, Deref)]
pub struct BuildingData(HashMap<BuildingId, BuildingSpec>);

impl BuildingData
{
    pub fn new(configs: &RootConfigs) -> Result<Self, String>
    {
        configs.get_type_from_file::<Self>("building_data")
    }

    pub(crate) fn validate(&self, resources: &ResourceData) -> Result<(), String>
    {
        for (id, spec) in self.iter() {
            spec.validate(id, self, resources)?;
        }

        Ok(())
    }

    pub fn get_tileselect_tiles(&self) -> Option<u16>
    {
        self.get("hq-1").and_then(|spec| match &spec.building_type {
            BuildingTypeSpec::Hq { total_tiles_allowed } => Some(*total_tiles_allowed),
            _ => None,
        })
    }
}

//-------------------------------------------------------------------------------------------------------------------
