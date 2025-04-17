use bevy::prelude::*;
use bevy_girk_game_fw::GameSender;

use crate::vis::*;
use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn handle_start_tileselect(ctx: Res<GameContext>, game_time: Res<GameTime>, mut sender: GameSender)
{
    let Some(remaining_ms) = ctx
        .duration_config()
        .select_remaining_ms(game_time.elapsed())
    else {
        return;
    };
    sender.send(GameMsg::TileSelectInfo { remaining_ms }, vis!(Global));
}

//-------------------------------------------------------------------------------------------------------------------

fn update_game_round(
    ctx: Res<GameContext>,
    game_time: Res<GameTime>,
    mut game_round: ResMut<GameRound>,
    mut sender: GameSender,
    mut events: EventWriter<RoundChange>,
)
{
    let Some((round, remaining_ms)) = ctx
        .duration_config()
        .round_and_remaining_ms(game_time.elapsed())
    else {
        return;
    };
    let prev = **game_round;
    game_round.0 = round;

    if prev != round {
        sender.send(GameMsg::RoundInfo { round, remaining_ms }, vis!(Global));
        events.send(RoundChange { prev, round });
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// The current game round.
#[derive(Resource, Default, Debug, Deref)]
pub(crate) struct GameRound(u32);

//-------------------------------------------------------------------------------------------------------------------

#[derive(Event, Debug)]
pub(crate) struct RoundChange
{
    #[allow(dead_code)]
    pub(crate) prev: u32,
    #[allow(dead_code)]
    pub(crate) round: u32,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct RoundUpdateSet;

//-------------------------------------------------------------------------------------------------------------------

/// Game tick plugin.
pub(crate) struct GameRoundPlugin;

impl Plugin for GameRoundPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<GameRound>()
            .add_event::<RoundChange>()
            .configure_sets(Update, RoundUpdateSet.in_set(GameSet::Play))
            .add_systems(OnEnter(GameState::TileSelect), handle_start_tileselect)
            .add_systems(Update, update_game_round.in_set(RoundUpdateSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
