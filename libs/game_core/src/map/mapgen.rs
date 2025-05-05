use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_girk_utils::Rand64;
use hexx::{Hex, HexLayout, HexOrientation};
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Checks if a tile is on the edge of the map using its index in a rectangular `hexx` grid.
fn is_edge_tile(map_dimension: i32, edge_buffer: i32, i: i32) -> bool
{
    let side_length = map_dimension * 2 + 1;
    let total_tiles = side_length * side_length;

    // left
    if i < edge_buffer * side_length {
        return true;
    }

    // top
    if i % side_length < edge_buffer {
        return true;
    }

    // bottom
    if i % side_length > (side_length - 1 - edge_buffer) {
        return true;
    }

    // right
    if i > total_tiles - edge_buffer * side_length {
        return true;
    }

    false
}

//-------------------------------------------------------------------------------------------------------------------

fn spawn_map_tile(
    c: &mut Commands,
    i: usize,
    coord: Hex,
    pos: Vec2,
    prng: f64,
    settings: &MapGenSettings,
    total: u64,
    freqs: &[(u16, TileType)],
) -> Entity
{
    let transform = Transform::from_xyz(pos.x, pos.y, 0.0);

    if is_edge_tile(settings.map_dimension, settings.edge_buffer as i32, i as i32) {
        return c
            .spawn((EdgeTile(coord), TileType::Mountain, MountainTile, transform))
            .id();
    }

    let mut acc: u64 = 0;
    for (freq, tile) in freqs.iter().copied() {
        acc += freq as u64;
        if (acc as f64 / total as f64) < prng {
            continue;
        }
        match tile {
            TileType::Mountain => {
                return c
                    .spawn((MapTile(coord), tile, MountainTile, transform))
                    .id()
            }
            TileType::Water => return c.spawn((MapTile(coord), tile, WaterTile, transform)).id(),
            TileType::Rocky => return c.spawn((MapTile(coord), tile, RockyTile, transform)).id(),
            TileType::Ore => return c.spawn((MapTile(coord), tile, OreTile, transform)).id(),
            TileType::Forest => return c.spawn((MapTile(coord), tile, ForestTile, transform)).id(),
            TileType::Grass => return c.spawn((MapTile(coord), tile, GrassTile, transform)).id(),
        }
    }

    Entity::PLACEHOLDER
}

//-------------------------------------------------------------------------------------------------------------------

fn generate_map(mut c: Commands, ctx: Res<GameContext>, settings: Res<MapGenSettings>)
{
    generate_map_impl(&mut c, map_gen_prng(ctx.seed), &settings);
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Serialize, Deserialize, Debug, Clone)]
pub struct MapGenSettings
{
    pub hex_scale: Vec2,
    /// Half-size of one side of the map square, in number of tiles.
    pub map_dimension: i32,
    /// Number of tile layers on the edge of the map containing untouchable boundary tiles.
    pub edge_buffer: u8,
    pub mountain_tile_frequency: u16,
    pub water_tile_frequency: u16,
    pub rocky_tile_frequency: u16,
    pub ore_tile_frequency: u16,
    pub forest_tile_frequency: u16,
    pub grass_tile_frequency: u16,
}

//-------------------------------------------------------------------------------------------------------------------

/// Stores information about the hex tile grid.
#[derive(Resource, Debug, Default)]
pub struct HexGrid
{
    pub tiles: HashMap<Hex, Entity>,
    pub entities: HashMap<Entity, Hex>,
    pub layout: HexLayout,
    pub dimension: i32,
}

//-------------------------------------------------------------------------------------------------------------------

pub fn generate_map_impl(c: &mut Commands, prng: u64, settings: &MapGenSettings)
{
    // prep tile generator
    let mut prng = Rand64::new("MAP GEN PRNG", prng as u128);

    // prep tile frequencies
    let freqs = [
        (settings.mountain_tile_frequency, TileType::Mountain),
        (settings.water_tile_frequency, TileType::Water),
        (settings.rocky_tile_frequency, TileType::Rocky),
        (settings.ore_tile_frequency, TileType::Ore),
        (settings.forest_tile_frequency, TileType::Forest),
        (settings.grass_tile_frequency, TileType::Grass),
    ];
    let freq_total: u64 = freqs.iter().map(|(freq, _)| *freq as u64).sum();

    // spawn the hex grid with sprites assigned to each hex
    let layout = HexLayout {
        orientation: HexOrientation::Flat,
        scale: settings.hex_scale,
        ..default()
    };
    let map_dim = settings.map_dimension;

    let mut tiles = HashMap::default();
    let mut entities = HashMap::default();
    let dim = map_dim * 2 + 1;
    tiles.reserve((dim * dim) as usize);
    entities.reserve((dim * dim) as usize);
    for (coord, entity) in hexx::shapes::flat_rectangle([-map_dim, map_dim, -map_dim, map_dim])
        .enumerate()
        .map(|(i, coord)| {
            let pos = layout.hex_to_world_pos(coord);
            let prng = prng.next() as f64 / u64::MAX as f64;
            let entity = spawn_map_tile(c, i, coord, pos, prng, settings, freq_total, &freqs);
            (coord, entity)
        })
    {
        tiles.insert(coord, entity);
        entities.insert(entity, coord);
    }
    c.insert_resource(HexGrid { tiles, entities, layout, dimension: map_dim });
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct MapGenPlugin;

impl Plugin for MapGenPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(PostStartup, generate_map);
    }
}

//-------------------------------------------------------------------------------------------------------------------
