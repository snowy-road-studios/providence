use bevy::prelude::*;
use utils_gui::{LayerIndex, SpriteLayerPlugin};

//-------------------------------------------------------------------------------------------------------------------

#[derive(Component, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum SpriteLayer
{
    /// We don't add SpriteLayer::Tiles to tiles because there are a lot of tiles. Instead, tiles are implicitly
    /// at the `0.0` layer.
    Tiles,
    TileOwnershipEffect,
    TileSelectEffect,
    /// Includes buildings, NPCs, etc.
    Things,
}

impl LayerIndex for SpriteLayer
{
    fn as_z_coordinate(&self) -> f32
    {
        match self {
            Self::Tiles => 0.0,
            Self::TileOwnershipEffect => 1.0,
            Self::TileSelectEffect => 2.0,
            Self::Things => 3.0,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct SpriteLayerImplPlugin;

impl Plugin for SpriteLayerImplPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(SpriteLayerPlugin::<SpriteLayer>::default());
    }
}

//-------------------------------------------------------------------------------------------------------------------
