use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy_121::*;
use bevy_replicon::prelude::Replicated;
use hexx::Hex;
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Component for all map tiles (including edge tiles).
#[derive(Component, Debug, Copy, Clone, Deref)]
#[component(immutable)]
pub struct MapTile(pub Hex);

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for edge tiles.
#[derive(Component, Debug, Copy, Clone)]
#[component(immutable)]
pub struct EdgeTile;

/// Marker component for tiles that have water proximity effects.
#[derive(Component, Debug, Copy, Clone)]
#[component(immutable)]
pub struct WaterTile;

/// Marker component for tiles that can be owned.
#[derive(Component, Debug, Copy, Clone)]
#[component(immutable)]
pub struct OwnableTile;

//-------------------------------------------------------------------------------------------------------------------

/// Component for tile meta entities.
///
/// Causes a [`ParentTile`] component to be auto-inserted.
///
/// Must not be inserted to any entity before the [`HexGrid`] resource has been inserted.
#[derive(Component, Debug, Copy, Clone, Serialize, Deserialize)]
#[component(immutable)]
#[component(on_insert = add_parent_tile)]
#[require(Replicated)]
pub struct TileMeta
{
    /// Tile this meta entity is attached to.
    pub tile: Hex,
}

fn add_parent_tile(mut w: DeferredWorld, context: HookContext)
{
    let meta = w.get::<TileMeta>(context.entity).unwrap();
    let grid = w
        .get_resource::<HexGrid>()
        .expect("HexGrid must be inserted before spawning TileMeta entities");
    let parent = grid
        .tiles
        .get(&meta.tile)
        .copied()
        .expect("TileMeta entities should have valid Hex values");
    w.commands()
        .entity(context.entity)
        .try_insert(ParentTile(parent));
}

//-------------------------------------------------------------------------------------------------------------------

/// Marker component for tile metas of tiles that can be selected by each player during tile select.
//TODO: on spawn, must include vis!(Client(client_id))
//TODO: when a tile is selected, despawn the SelectedTile entity and spawn a TileOwner entity; when deselected,
//despawn the TileOwner and spawn a SelectedTile
#[derive(Component, Debug, Copy, Clone, Serialize, Deserialize)]
#[component(immutable)]
pub struct SelectableTile
{
    pub client: u64,
}

//-------------------------------------------------------------------------------------------------------------------

/// Current claims on an unowned tile.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct TileClaims
{
    /// Current claimants.
    claimants: HashSet<u64>,
    /// Claiming age, used when setting claim costs.
    age: u16,
}

impl TileClaims
{
    pub(crate) fn _new(client: u64) -> Self
    {
        let claimants = HashSet::from_iter([client]);
        Self { claimants, age: 0 }
    }

    pub(crate) fn _add_claimant(&mut self, client: u64) -> bool
    {
        self.claimants.insert(client)
    }

    pub(crate) fn _remove_claimant(&mut self, client: u64) -> bool
    {
        self.claimants.remove(&client)
    }

    /// Advances the claiming age and returns the previous rounds' claimants.
    pub(crate) fn _next_round(&mut self) -> HashSet<u64>
    {
        self.age += 1;
        std::mem::take(&mut self.claimants)
    }

    pub fn claimants(&self) -> &HashSet<u64>
    {
        &self.claimants
    }

    pub fn age(&self) -> u16
    {
        self.age
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// The owner of a tile.
#[derive(Component, Debug, Copy, Clone, Deref, Serialize, Deserialize)]
#[component(immutable)]
pub struct TileOwner(pub u64);

//-------------------------------------------------------------------------------------------------------------------

/// Custom 1:1 relationship to attach tile meta info to tile entities.
///
/// We use a 1:1 relationship for ease of traversing the connection. Tile meta info is stored separately
/// so it can be replicated without replicating the entire map.
///
/// See [`TileMeta`].
#[derive(AsymmetricOneToOne, Deref)]
#[target(AttachedMeta)]
pub struct ParentTile(Entity);

/// Meta tile entity attached to this tile entity.
///
/// See [`TileMeta`].
#[derive(AsymmetricOneToOne, Deref)]
#[target(ParentTile)]
pub struct AttachedMeta(Entity);

//-------------------------------------------------------------------------------------------------------------------
