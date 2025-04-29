use bevy::ecs::intern::Interned;
use bevy::ecs::schedule::ScheduleLabel;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(Resource, Default)]
struct ReinitContext
{
    callbacks: HashMap<Interned<dyn ScheduleLabel>, Vec<fn(&mut World)>>,
}

//-------------------------------------------------------------------------------------------------------------------

fn reinit_resource_fn<T: Resource + FromWorld>(world: &mut World)
{
    let res = T::from_world(world);
    world.insert_resource(res);
}

//-------------------------------------------------------------------------------------------------------------------

fn handle_reinits(schedule: Interned<dyn ScheduleLabel>) -> impl IntoSystem<(), (), ()>
{
    IntoSystem::into_system(move |world: &mut World| {
        let mut ctx = world.get_resource_or_init::<ReinitContext>();
        let ctx_callbacks = ctx.callbacks.entry(schedule).or_default();
        let mut callbacks = std::mem::take(ctx_callbacks);

        for callback in callbacks.iter() {
            (*callback)(world);
        }

        let mut ctx = world.get_resource_or_init::<ReinitContext>();
        let ctx_callbacks = ctx.callbacks.entry(schedule).or_default();
        callbacks.extend(ctx_callbacks.drain(..));
        *ctx_callbacks = callbacks;
    })
}

//-------------------------------------------------------------------------------------------------------------------

fn reinit_resource<T: Resource + FromWorld>(app: &mut App, schedule: impl ScheduleLabel)
{
    app.init_resource::<T>();

    let mut ctx = app.world_mut().get_resource_or_init::<ReinitContext>();
    let interned_schedule = schedule.intern();
    let ctx_callbacks = ctx.callbacks.entry(interned_schedule).or_default();
    ctx_callbacks.push(reinit_resource_fn::<T>);

    if ctx_callbacks.len() == 1 {
        app.add_systems(schedule, handle_reinits(interned_schedule));
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// [`App`] extension for reinitializing resources automatically on state changes.
pub trait ReinitAppExt
{
    /// Initializes resource `T` and reinitializes it when entering `state`.
    fn reinit_resource_on_enter<T: Resource + FromWorld>(&mut self, state: impl States) -> &mut Self;
    /// Initializes resource `T` and reinitializes it when exiting `state`.
    fn reinit_resource_on_exit<T: Resource + FromWorld>(&mut self, state: impl States) -> &mut Self;
}

impl ReinitAppExt for App
{
    fn reinit_resource_on_enter<T: Resource + FromWorld>(&mut self, state: impl States) -> &mut Self
    {
        reinit_resource::<T>(self, OnEnter(state));
        self
    }
    fn reinit_resource_on_exit<T: Resource + FromWorld>(&mut self, state: impl States) -> &mut Self
    {
        reinit_resource::<T>(self, OnExit(state));
        self
    }
}

//-------------------------------------------------------------------------------------------------------------------
