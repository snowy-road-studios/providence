use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_girk_utils::Sender;
use bevy_renet2::prelude::RenetClient;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn add_command_button<R: CobwebResult, M>(
    h: &mut UiSceneHandle,
    name: &'static str,
    callback: impl IntoSystem<(), R, M> + Send + Sync + 'static,
)
{
    h.spawn_scene(("client.game.settings", "command_button"), |h| {
        h.get("text").update_text(name);
        h.on_pressed(callback);
    });
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn build_settings_commands_section(h: &mut UiSceneHandle)
{
    // Jump to the next round.
    add_command_button(h, "Next Round", |sender: Res<Sender<CommandInput>>| {
        let _ = sender.send(CommandInput::NextRound);
    });
    // Disconnects the renet2 client.
    add_command_button(h, "Disconnect", |client: Option<ResMut<RenetClient>>| {
        client.result()?.disconnect();
        DONE
    });
    // Forces the game to end.
    add_command_button(
        h,
        "End Game",
        |mut c: Commands, sender: Res<Sender<CommandInput>>, mut state: ResMut<NextState<ClientState>>| {
            let _ = sender.send(CommandInput::EndGame);
            state.set(ClientState::End);
            c.react().broadcast(CloseSettings);
        },
    );
}

//-------------------------------------------------------------------------------------------------------------------
