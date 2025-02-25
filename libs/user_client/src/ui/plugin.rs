use bevy::prelude::*;
use bevy_cobweb_ui::prelude::*;
use bevy_girk_client_fw::ClientAppState;

use super::*;

//-------------------------------------------------------------------------------------------------------------------

fn build_ui(mut c: Commands, mut s: SceneBuilder)
{
    c.ui_root().spawn_scene(("ui.user", "main"), &mut s, |h| {
        h.insert(StateScoped(ClientAppState::Client));

        let content_id = h.get_entity("content")?;

        h.edit("sidebar", |h| {
            h.spawn_scene(("ui.user.sidebar", "sidebar"), |h| build_sidebar(h, content_id));
        });

        OK
    });
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct UiPlugin;

impl Plugin for UiPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_systems(OnEnter(ClientAppState::Client), build_ui)
            // ui plugins
            .add_plugins(UiSidebarPlugin)
            .add_plugins(UiReconnectingPlugin)
            .add_plugins(UiAckLobbyPopupPlugin)
            // ui menu sections
            .add_plugins(UiHomeSectionPlugin)
            .add_plugins(UiPlaySectionPlugin)
            .add_plugins(UiSettingsSectionPlugin)
            // load content
            .load("user_client/main.cob");
    }
}

//-------------------------------------------------------------------------------------------------------------------
