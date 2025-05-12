use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_girk_client_fw::ClientFwState;
use game_core::*;
use utils::*;
use wiring_game_instance::ClientContext;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn spawn_map(mut c: Commands, ctx: Res<ClientContext>, settings: Res<MapGenSettings>)
{
    generate_map_impl(&mut c, ctx.map_gen_prng, &settings, false);
    c.react().broadcast(MapGenerated);
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event broadcasted when the map has been generated on the client.
pub struct MapGenerated;

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct MapgenPlugin;

impl Plugin for MapgenPlugin
{
    fn build(&self, app: &mut App)
    {
        app.register_required_components_with::<TileType, StateScoped<ClientAppState>>(|| {
            StateScoped(ClientAppState::Game)
        })
        .reinit_resource_on_enter::<HexGrid>(ClientAppState::Client)
        .add_systems(
            // We wait until connecting is done so spawning the map doesn't block networking updates.
            OnExit(ClientFwState::Connecting),
            spawn_map.run_if(in_state(ClientAppState::Game)),
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------
