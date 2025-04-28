use bevy::ecs::schedule::ScheduleLabel;
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

fn try_end_round(mut c: Commands, ctx: Res<GameContext>, game_time: Res<GameTime>, game_round: Res<GameRound>)
{
    let Some((round, _remaining_ms)) = ctx
        .duration_config()
        .round_and_remaining_ms(game_time.elapsed())
    else {
        return;
    };

    let prev = **game_round;
    if prev == round || prev == 0 {
        return;
    }

    c.queue(|w: &mut World| w.run_schedule(RoundEnd));
}

//-------------------------------------------------------------------------------------------------------------------

fn try_start_round(
    mut c: Commands,
    ctx: Res<GameContext>,
    game_time: Res<GameTime>,
    mut game_round: ResMut<GameRound>,
    mut sender: GameSender,
)
{
    let Some((round, remaining_ms)) = ctx
        .duration_config()
        .round_and_remaining_ms(game_time.elapsed())
    else {
        return;
    };

    let prev = **game_round;
    if prev == round || round > ctx.duration_config().num_rounds {
        return;
    }

    game_round.0 = round;
    sender.send(GameMsg::RoundInfo { round, remaining_ms }, vis!(Global));
    c.queue(|w: &mut World| w.run_schedule(RoundStart));
}

//-------------------------------------------------------------------------------------------------------------------

/// The current game round.
///
/// Updates between the [`RoundEnd`] and [`RoundStart`] schedules.
#[derive(Resource, Default, Debug, Deref)]
pub(crate) struct GameRound(u32);

//-------------------------------------------------------------------------------------------------------------------

/// Schedule that runs at the end of a round.
#[derive(ScheduleLabel, Debug, Hash, Eq, PartialEq, Clone)]
pub(crate) struct RoundEnd;

//-------------------------------------------------------------------------------------------------------------------

/// Schedule that runs at the beginning of a round.
#[derive(ScheduleLabel, Debug, Hash, Eq, PartialEq, Clone)]
pub(crate) struct RoundStart;

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct RoundEndSet;

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) struct RoundStartSet;

//-------------------------------------------------------------------------------------------------------------------

/// Game tick plugin.
pub(crate) struct GameRoundPlugin;

impl Plugin for GameRoundPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_resource::<GameRound>()
            .init_schedule(RoundEnd)
            .init_schedule(RoundStart)
            .configure_sets(Update, (RoundEndSet, RoundStartSet).in_set(GameSet::Play))
            .add_systems(OnEnter(GameState::TileSelect), handle_start_tileselect)
            .add_systems(Update, try_end_round.in_set(RoundEndSet))
            .add_systems(Update, try_start_round.in_set(RoundStartSet));
    }
}

//-------------------------------------------------------------------------------------------------------------------
