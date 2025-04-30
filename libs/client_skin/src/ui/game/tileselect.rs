use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn build_overlay(mut c: Commands, mut s: SceneBuilder)
{
    c.ui_root()
        .spawn_scene(("client.game.tileselect", "overlay"), &mut s, |h| {
            h.insert(StateScoped(ClientState::TileSelect));

            h.get("text")
                // TODO: get current number of claimed tiles from query
                .update_on(
                    broadcast::<AppUpdateEnd>(),
                    |id: TargetId, mut e: TextEditor, buildings: Res<BuildingData>| {
                        write_text!(e, *id, "SELECT TILES: 0/{}", buildings.hq[0].ownable_tiles);
                    },
                );
        });
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct GameUiTileSelectPlugin;

impl Plugin for GameUiTileSelectPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnEnter(ClientState::TileSelect), build_overlay);
    }
}

//-------------------------------------------------------------------------------------------------------------------
