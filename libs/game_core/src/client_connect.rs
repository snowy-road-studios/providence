use bevy::prelude::*;
use bevy_replicon::shared::backend::connected_client::NetworkId;

use crate::vis::*;
use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_client_connect(
    event: Trigger<OnAdd, NetworkId>,
    ids: Query<&NetworkId>,
    ctx: Res<GameContext>,
    state: Res<State<GameState>>,
    time: Res<GameTime>,
    mut sender: GameSender,
)
{
    let Ok(id) = ids.get(event.target()) else { return };
    let client_id = id.get();

    match **state {
        GameState::Startup | GameState::Init => (),
        GameState::TileSelect => {
            if let Some(remaining_ms) = ctx.duration_config().select_remaining_ms(time.elapsed()) {
                sender.send(GameMsg::TileSelectInfo { remaining_ms }, vis!(Client(client_id)));
            }
        }
        GameState::Play => {
            if let Some((round, remaining_ms)) = ctx.duration_config().round_and_remaining_ms(time.elapsed()) {
                sender.send(GameMsg::RoundInfo { round, remaining_ms }, vis!(Client(client_id)));
            }
        }
        GameState::End => (),
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct ClientConnectPlugin;

impl Plugin for ClientConnectPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_observer(handle_client_connect);
    }
}

//-------------------------------------------------------------------------------------------------------------------
