use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_girk_client_instance::{ClientInstanceCommand, LocalGameManager};

use super::*;
use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn add_menu_button(
    h: &mut UiSceneHandle,
    content_id: Entity,
    name: &str,
    scene: impl Into<SceneRef>,
    callback: fn(&mut UiSceneHandle),
) -> Entity
{
    let mut entity = Entity::PLACEHOLDER;
    h.spawn_scene(("client.game.settings", "menu_button"), |h| {
        entity = h.id();
        h.get("text").update_text(name);
        let scene: SceneRef = scene.into();
        h.on_select(move |mut c: Commands, mut s: SceneBuilder| {
            c.get_entity(content_id).result()?.despawn_descendants();
            c.ui_builder(content_id)
                .spawn_scene(scene.clone(), &mut s, callback);
            DONE
        });
    });

    entity
}

//-------------------------------------------------------------------------------------------------------------------

fn build_settings(mut c: Commands, mut s: SceneBuilder, localgame: Res<LocalGameManager>)
{
    c.ui_root()
        .spawn_scene(("client.game.settings", "settings_popup"), &mut s, |h| {
            h.insert(StateScoped(ClientAppState::Game));
            h.apply(DisplayControl::Hide);

            h.reactor(broadcast::<OpenSettings>(), |id: TargetId, mut c: Commands| {
                c.get_entity(*id).result()?.apply(DisplayControl::Show);
                DONE
            });
            h.reactor(broadcast::<CloseSettings>(), |id: TargetId, mut c: Commands| {
                c.get_entity(*id).result()?.apply(DisplayControl::Hide);
                DONE
            });
            h.reactor(
                broadcast::<ToggleSettings>(),
                |id: TargetId, mut c: Commands, control: Query<&DisplayControl>| {
                    let mut ec = c.get_entity(*id).result()?;
                    match control.get(*id).copied() {
                        Ok(DisplayControl::Show) => {
                            ec.apply(DisplayControl::Hide);
                        }
                        Err(_) | Ok(DisplayControl::Hide) => {
                            ec.apply(DisplayControl::Show);
                        }
                    }
                    DONE
                },
            );

            let content_id = h.get_entity("window::main::content")?;

            h.edit("window::main::sidebar", |h| {
                let game_section_id = add_menu_button(
                    h,
                    content_id,
                    "Game",
                    ("client.game.settings", "game_section"),
                    build_settings_game_section,
                );
                add_menu_button(
                    h,
                    content_id,
                    "Hotkeys",
                    ("client.game.settings", "game_section"),
                    build_settings_hotkeys_section,
                );
                add_menu_button(
                    h,
                    content_id,
                    "Video",
                    ("client.game.settings", "video_section"),
                    build_settings_video_section,
                );
                add_menu_button(
                    h,
                    content_id,
                    "Audio",
                    ("client.game.settings", "audio_section"),
                    build_settings_audio_section,
                );
                #[cfg(feature = "dev")]
                add_menu_button(
                    h,
                    content_id,
                    "Dev",
                    ("client.game.settings", "dev_section"),
                    build_settings_dev_section,
                );

                // Initialize menu.
                h.react().entity_event(game_section_id, Select);
            });

            h.edit("window::footer::quit_button", |h| {
                if localgame.is_running() {
                    h.on_pressed(|mut c: Commands| {
                        c.queue(ClientInstanceCommand::Abort);
                    });
                } else {
                    let id = h.id();
                    h.commands().get_entity(id).result()?.despawn_recursive();
                }
                DONE
            });
            h.get("window::footer::done_button")
                .on_pressed(|mut c: Commands| c.react().broadcast(CloseSettings));

            OK
        });
}

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event to broadcast to open the settings popup.
pub(crate) struct OpenSettings;

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event to broadcast to close the settings popup.
pub(crate) struct CloseSettings;

//-------------------------------------------------------------------------------------------------------------------

/// Reactive event to broadcast to toggle the settings popup.
pub(crate) struct ToggleSettings;

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct GameUiSettingsPlugin;

impl Plugin for GameUiSettingsPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnEnter(ClientState::TileSelect), build_settings);
    }
}

//-------------------------------------------------------------------------------------------------------------------
