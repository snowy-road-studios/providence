use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ResourceCost
{
    #[serde(default)]
    pub gold: u64,
    #[serde(default)]
    pub food: u64,
    #[serde(default)]
    pub wood: u64,
    #[serde(default)]
    pub stone: u64,
    #[serde(default)]
    pub iron: u64,
}
