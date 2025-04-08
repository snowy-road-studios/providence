use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_girk_client_fw::ClientAppState;
use bevy_renet2::prelude::RenetClient;
use client_core::ClientState;
use wiring_game_instance::{ClientContext, ClientType};

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn edit_header(mut h: UiSceneHandle)
{
    h.get("name_shim::name")
        .update(|id: TargetId, mut e: TextEditor, context: Res<ClientContext>| {
            match context.client_type() {
                ClientType::Player => write_text!(e, *id, "player{}", context.id()),
                ClientType::Watcher => write_text!(e, *id, "watcher{}", context.id()),
            };
        });
    h.get("fps::text").update_on(
        resource_mutation::<FpsTracker>(),
        |id: TargetId, mut next_time: Local<u64>, mut e: TextEditor, fps: ReactRes<FpsTracker>| {
            // only refresh once per second
            let current_time = fps.current_time().as_secs();
            if current_time < *next_time {
                return;
            }
            *next_time = current_time + 1;

            write_text!(e, *id, "FPS: {}", fps.fps());
        },
    );
}

//-------------------------------------------------------------------------------------------------------------------

fn edit_footer(mut h: UiSceneHandle)
{
    // Disconnect button. Lets you test in-game disconnects.
    h.get("disconnect_button")
        .update(
            |id: TargetId, mut c: Commands, ps: PseudoStateParam, context: Res<ClientContext>| {
                if context.client_type() != ClientType::Player {
                    ps.try_disable(&mut c, *id);
                }
            },
        )
        .on_pressed(|mut client: ResMut<RenetClient>| {
            client.disconnect();
        });
}

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: SceneBuilder)
{
    let scene = ("ui.skin.game", "game");
    c.ui_root().spawn_scene(scene, &mut s, |h| {
        h.insert(StateScoped(ClientAppState::Game));

        edit_header(h.get("header"));
        edit_footer(h.get("footer"));
    });
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct GameUiPlugin;

impl Plugin for GameUiPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnEnter(ClientState::Play), build_ui);
    }
}

//-------------------------------------------------------------------------------------------------------------------
