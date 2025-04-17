use bevy::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) fn handle_dev_input(In((_player_entity, input)): In<(Entity, DevInput)>, world: &mut World)
{
    match input {
        DevInput::EndGame => {
            world
                .resource_mut::<NextState<GameState>>()
                .set(GameState::End);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
