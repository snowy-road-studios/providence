use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_girk_utils::Sender;
use bevy_renet2::prelude::RenetClient;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn add_dev_button<R: CobwebResult, M>(
    h: &mut UiSceneHandle,
    name: &'static str,
    callback: impl IntoSystem<(), R, M> + Send + Sync + 'static,
)
{
    h.spawn_scene(("client.game.settings", "dev_button"), |h| {
        h.get("text").update_text(name);
        h.on_pressed(callback);
    });
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn build_settings_dev_section(h: &mut UiSceneHandle)
{
    // Disconnects the renet2 client.
    add_dev_button(h, "Disconnect", |mut client: ResMut<RenetClient>| client.disconnect());
    // Forces the game to end.
    add_dev_button(h, "End Game", |sender: Res<Sender<DevInput>>| {
        let _ = sender.send(DevInput::EndGame);
    });
}

//-------------------------------------------------------------------------------------------------------------------
