use std::borrow::{Borrow, Cow};

use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use serde::Deserialize;

//-------------------------------------------------------------------------------------------------------------------

/// Component with the canonical ID of a type of resource.
#[derive(Component, Debug, Deserialize, Clone, Eq, PartialEq, Hash, Reflect)]
#[reflect(Hash, Deserialize)]
#[component(immutable)]
pub struct ResourceId(Cow<'static, str>);

impl ResourceId
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

impl Borrow<str> for ResourceId
{
    fn borrow(&self) -> &str
    {
        self.get()
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Note: Does not include gold, which is a special asset that is not treated as a resource that can be collected
/// and consumed.
#[derive(Resource, Debug, Deserialize, Clone, Deref)]
pub struct ResourceData(HashSet<ResourceId>);

impl ResourceData
{
    pub fn new() -> Self
    {
        let mut set = HashSet::default();
        set.insert(ResourceId::new("food"));
        set.insert(ResourceId::new("wood"));
        set.insert(ResourceId::new("stone"));
        set.insert(ResourceId::new("ore"));
        set.insert(ResourceId::new("hyperium"));

        Self(set)
    }

    pub(crate) fn validate(&self) -> Result<(), String>
    {
        Ok(())
    }
}

//-------------------------------------------------------------------------------------------------------------------
