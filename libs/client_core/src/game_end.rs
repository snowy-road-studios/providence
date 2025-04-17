use std::time::Duration;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_girk_client_fw::{ClientFwConfig, ClientFwState};
use bevy_girk_game_fw::GameOverReport;
use game_core::ProvGameOverReport;
use wiring_game_instance::ClientContext;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Manually detect game end instead of waiting for the server to send GameState::End.
fn detect_game_end(
    config: Res<ClientContext>,
    timer: Res<RoundTimer>,
    mut fw_state: ResMut<NextState<ClientFwState>>,
    mut state: ResMut<NextState<ClientState>>,
)
{
    let dur_config = config.duration_config();
    if timer.round() < dur_config.num_rounds {
        return;
    }
    if timer.remaining_time() != Duration::default() {
        return;
    }

    fw_state.set(ClientFwState::End);
    state.set(ClientState::End);
}

//-------------------------------------------------------------------------------------------------------------------

fn end_if_game_over_report(
    event: BroadcastEvent<GameOverReport>,
    config: Option<Res<ClientFwConfig>>,
    fw_state: Option<ResMut<NextState<ClientFwState>>>,
    state: Option<ResMut<NextState<ClientState>>>,
)
{
    let Some(report) = event.read().get::<ProvGameOverReport>() else { return };
    let (Some(config), Some(mut next_fw_state), Some(mut next_state)) = (config, fw_state, state) else { return };
    if report.game_id != config.game_id() {
        return;
    }

    next_fw_state.set(ClientFwState::End);
    next_state.set(ClientState::End);
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct GameEndPlugin;

impl Plugin for GameEndPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(Update, detect_game_end.run_if(in_state(ClientState::Play)))
            .add_reactor(broadcast::<GameOverReport>(), end_if_game_over_report);
    }
}

//-------------------------------------------------------------------------------------------------------------------
