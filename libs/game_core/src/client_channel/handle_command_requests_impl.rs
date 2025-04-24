use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_command_input(In((_player_entity, input)): In<(Entity, CommandInput)>, world: &mut World)
{
    match input {
        CommandInput::EndGame => {
            world
                .resource_mut::<NextState<GameState>>()
                .set(GameState::End);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
