use bevy::platform::collections::HashMap;
use bevy::prelude::Deref;
use serde::Deserialize;

use super::ResourceId;

//-------------------------------------------------------------------------------------------------------------------

/// Note: does not include gold, which cannot be spent to construct things.
#[derive(Deserialize, Debug, Clone, Deref)]
pub struct ResourceCost(pub HashMap<ResourceId, u64>);

//-------------------------------------------------------------------------------------------------------------------
