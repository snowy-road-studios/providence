use bevy::prelude::*;
use hexx::Hex;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[component(immutable)]
pub enum TileType
{
    Mountain,
    Water,
    Rocky,
    Ore,
    Forest,
    Grass,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug, Copy, Clone, Deref)]
#[component(immutable)]
pub struct EdgeTile(pub Hex);

#[derive(Component, Debug, Copy, Clone, Deref)]
#[component(immutable)]
pub struct MapTile(pub Hex);

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Debug, Copy, Clone)]
#[component(immutable)]
pub struct MountainTile;

#[derive(Component, Debug, Copy, Clone)]
#[component(immutable)]
pub struct WaterTile;

#[derive(Component, Debug, Copy, Clone)]
#[component(immutable)]
pub struct RockyTile;

#[derive(Component, Debug, Copy, Clone)]
#[component(immutable)]
pub struct OreTile;

#[derive(Component, Debug, Copy, Clone)]
#[component(immutable)]
pub struct ForestTile;

#[derive(Component, Debug, Copy, Clone)]
#[component(immutable)]
pub struct GrassTile;

//-------------------------------------------------------------------------------------------------------------------
