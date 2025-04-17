use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

/// Game plugin.
///
/// Depends on [`GameFwPlugin`], which should be added by the game factory.
pub struct GamePlugin;

impl Plugin for GamePlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_plugins(GameReplicationPlugin)
            .add_plugins(GameTimePlugin)
            .add_plugins(GameSetsPlugin)
            .add_plugins(GameSetupPlugin)
            .add_plugins(GameStatePlugin)
            .add_plugins(GameRoundPlugin)
            .add_plugins(ClientConnectPlugin)
            .configure_sets(
                Update,
                (TimeUpdateSet, GameStateUpdateSet, RoundUpdateSet)
                    .chain()
                    .in_set(GameLogicSet::Admin),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------
