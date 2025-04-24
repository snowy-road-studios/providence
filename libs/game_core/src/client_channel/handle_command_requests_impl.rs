use std::time::Duration;

use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_command_input(In((_player_entity, input)): In<(Entity, CommandInput)>, world: &mut World)
{
    match input {
        CommandInput::NextRound => {
            let gametime = world.resource::<GameTime>().elapsed();
            let duration_config = world.resource::<GameContext>().duration_config();
            let remaining_ms = if let Some(remaining_ms) = duration_config.select_remaining_ms(gametime) {
                remaining_ms
            } else if let Some((_, remaining_ms)) = duration_config.round_and_remaining_ms(gametime) {
                remaining_ms
            } else {
                return;
            };
            world
                .resource_mut::<GameTime>()
                .add_timeskip(Duration::from_millis(remaining_ms as u64 + 1));
        }
        CommandInput::EndGame => {
            world
                .resource_mut::<NextState<GameState>>()
                .set(GameState::End);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
