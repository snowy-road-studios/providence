use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn edit_header(h: &mut UiSceneHandle)
{
    h.get("name_shim::name")
        .update(|id: TargetId, mut e: TextEditor, context: Res<ClientContext>| {
            match context.client_type() {
                ClientType::Player => write_text!(e, *id, "player{}", context.id()),
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

fn build_hud(mut c: Commands, mut s: SceneBuilder)
{
    c.ui_root()
        .spawn_scene(("client.game.hud", "hud"), &mut s, |h| {
            h.insert(StateScoped(ClientAppState::Game));

            h.edit("top", edit_header);
        });
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct GameUiHudPlugin;

impl Plugin for GameUiHudPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnEnter(ClientState::TileSelect), build_hud);
    }
}

//-------------------------------------------------------------------------------------------------------------------
