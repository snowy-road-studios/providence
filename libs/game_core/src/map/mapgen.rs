use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use bevy_girk_utils::Rand64;
use hexx::{Hex, HexLayout, HexOrientation};
use serde::{Deserialize, Serialize};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

struct MapGenTile<'a>
{
    id: TileId,
    spec: &'a TileSpec,
    rng_cutoff: f64,
}

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

fn try_spawn_map_tile(
    c: &mut Commands,
    i: usize,
    coord: Hex,
    pos: Vec2,
    prng: f64,
    settings: &MapGenSettings,
    tiles: &[MapGenTile],
    edge_tile: &MapGenTile,
    ignore_edge_tiles: bool,
) -> Option<Entity>
{
    let transform = Transform::from_xyz(pos.x, pos.y, 0.0);

    if is_edge_tile(settings.map_dimension, settings.edge_buffer as i32, i as i32) {
        if ignore_edge_tiles {
            return None;
        } else {
            let core = (MapTile(coord), edge_tile.id.clone(), transform);
            let ec = c.spawn((EdgeTile, core));
            return Some(ec.id());
        }
    }

    for tile in tiles.iter() {
        if tile.rng_cutoff < prng {
            continue;
        }
        let core = (MapTile(coord), tile.id.clone(), transform);
        let ec = match (tile.spec.is_ownable, tile.spec.is_water_tile) {
            (true, true) => c.spawn((OwnableTile, WaterTile, core)),
            (true, false) => c.spawn((OwnableTile, core)),
            (false, true) => c.spawn((WaterTile, core)),
            (false, false) => c.spawn(core),
        };
        return Some(ec.id());
    }

    // Fall back to edge tile if something went wrong.
    Some(
        c.spawn((MapTile(coord), edge_tile.id.clone(), transform))
            .id(),
    )
}

//-------------------------------------------------------------------------------------------------------------------

fn generate_map(mut c: Commands, ctx: Res<GameContext>, settings: Res<MapGenSettings>, tile_data: Res<TileData>)
{
    // Ignores edge tiles since the game server doesn't use them.
    generate_map_impl(&mut c, map_gen_prng(ctx.seed), &settings, &tile_data, true);
}

//-------------------------------------------------------------------------------------------------------------------

/// Settings for generating maps.
///
/// Loaded from config file.
#[derive(Resource, Serialize, Deserialize, Debug, Clone)]
pub struct MapGenSettings
{
    pub hex_scale: Vec2,
    /// Half-size of one side of the map square, in number of tiles.
    pub map_dimension: i32,
    /// Number of tile layers on the edge of the map containing untouchable boundary tiles.
    pub edge_buffer: u8,
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

/// Set `ignore_edge_tiles = true` if edge tile entities are not needed (e.g. the server).
pub fn generate_map_impl(
    c: &mut Commands,
    prng: u64,
    settings: &MapGenSettings,
    tile_data: &TileData,
    ignore_edge_tiles: bool,
)
{
    // prep tile generator
    let mut prng = Rand64::new("MAP GEN PRNG", prng as u128);

    // prep tile frequencies
    let mut tiles: Vec<MapGenTile> = tile_data
        .iter()
        .scan(0, |factor_sum, (id, spec)| {
            *factor_sum += spec.mapgen_factor;
            Some(MapGenTile { id: id.clone(), spec, rng_cutoff: *factor_sum as f64 })
        })
        .collect();
    let factor_total: f64 = tiles
        .last()
        .map(|last| last.rng_cutoff)
        .unwrap_or(1.0)
        .max(1.0);
    tiles.iter_mut().for_each(|tile| {
        tile.rng_cutoff /= factor_total;
    });
    let edge_tile = tiles
        .iter()
        .find(|tile| tile.spec.is_edge_tile)
        .expect("should be one edge tile type");

    // spawn the hex grid with sprites assigned to each hex
    let layout = HexLayout {
        orientation: HexOrientation::Flat,
        scale: settings.hex_scale,
        ..default()
    };
    let map_dim = settings.map_dimension;
    let dim = map_dim * 2 + 1;
    let map_size = (dim * dim) as usize;

    let mut tile_map = HashMap::default();
    let mut entities = HashMap::default();
    tile_map.reserve(map_size);
    entities.reserve(map_size);

    for (i, coord) in hexx::shapes::flat_rectangle([-map_dim, map_dim, -map_dim, map_dim]).enumerate() {
        let pos = layout.hex_to_world_pos(coord);
        let prng = prng.next() as f64 / u64::MAX as f64;
        let Some(entity) =
            try_spawn_map_tile(c, i, coord, pos, prng, settings, &tiles, edge_tile, ignore_edge_tiles)
        else {
            continue;
        };

        tile_map.insert(coord, entity);
        entities.insert(entity, coord);
    }
    c.insert_resource(HexGrid { tiles: tile_map, entities, layout, dimension: map_dim });
}

//-------------------------------------------------------------------------------------------------------------------

/// Expects [`MapGenSettings`] was inserted by the game factory.
pub(super) struct MapGenPlugin;

impl Plugin for MapGenPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(PostStartup, generate_map);
    }
}

//-------------------------------------------------------------------------------------------------------------------
