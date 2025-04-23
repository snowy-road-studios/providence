use bevy::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_girk_client_fw::ClientAppState;
use bevy_girk_client_instance::ClientInstanceCommand;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn game_over_screen(mut c: Commands, mut s: SceneBuilder)
{
    let scene = ("client.gameover", "gameover");
    c.ui_root().spawn_scene(scene, &mut s, |h| {
        h.insert(StateScoped(ClientAppState::Game));

        h.get("end_button").on_pressed(|mut c: Commands| {
            c.queue(ClientInstanceCommand::End);
        });
    });
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct GameOverPlugin;

impl Plugin for GameOverPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnEnter(ClientState::End), game_over_screen);
    }
}

//-------------------------------------------------------------------------------------------------------------------
