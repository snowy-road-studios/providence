use bevy::prelude::*;
use game_core::*;

//-------------------------------------------------------------------------------------------------------------------

/// Prepare a game app core.
///
/// Depends on game framework.
pub fn prepare_game_app_core(game_app: &mut App, game_initializer: ProvGameInitializer, game_data: GameData)
{
    game_app
        .add_plugins(GamePlugin)
        .insert_resource(game_initializer);
    game_data.insert(game_app.world_mut());
}

//-------------------------------------------------------------------------------------------------------------------
