use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use wiring_backend::MAX_LOBBY_PLAYERS;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn build_make_lobby_popup(_: &ActivateMakeLobbyPopup, h: &mut UiSceneHandle)
{
    tracing::trace!("building make lobby popup");

    // Reactors for auto-closing the popup.
    h.reactor(
        broadcast::<RequestEnded<MakeLobby>>(),
        |//
            id: TargetId,
            event: BroadcastEvent<RequestEnded<MakeLobby>>,
            mut c: Commands//
        |
        {
            match event.try_read()? {
                RequestEnded::Success => {
                    tracing::info!("MakeLobby request succeeded");
                    c.get_entity(*id)?.despawn();
                }
                RequestEnded::Failure => {
                    tracing::warn!("MakeLobby request failed");
                }
                _ => ()
            }
            DONE
        },
    );
    h.reactor(broadcast::<MadeLocalLobby>(), |id: TargetId, mut c: Commands| {
        c.get_entity(*id)?.despawn();
        DONE
    });

    // Window
    let popup_id = h.id();
    let mut h = h.get("window");

    // Form fields
    h.edit("content::grid::password_field", |_| {
        // does nothing yet
    });
    h.edit("content::grid::max_players_field", |h| {
        h.get("text").update_on(
            resource_mutation::<MakeLobbyData>(),
            |id: TargetId, mut e: TextEditor, data: ReactRes<MakeLobbyData>| {
                write_text!(e, *id, "{}", data.config.max_players);
            },
        );
        h.get("buttons::add_player_button")
            .on_pressed(|mut c: Commands, mut data: ReactResMut<MakeLobbyData>| {
                let data = data.get_mut(&mut c);
                data.config.max_players += 1;
                data.config.max_players = data.config.max_players.min(MAX_LOBBY_PLAYERS);
            })
            .enable_if(
                resource_mutation::<MakeLobbyData>(),
                |_: TargetId, data: ReactRes<MakeLobbyData>| data.config.max_players < MAX_LOBBY_PLAYERS,
            );
        h.get("buttons::remove_player_button")
            .on_pressed(|mut c: Commands, mut data: ReactResMut<MakeLobbyData>| {
                let max = data.config.max_players;
                data.get_mut(&mut c).config.max_players = max.saturating_sub(1).max(1);
            })
            .enable_if(
                resource_mutation::<MakeLobbyData>(),
                |_: TargetId, data: ReactRes<MakeLobbyData>| data.config.max_players > 1,
            );
    });
    h.edit("content::grid::join_as_field", |h| {
        h.get("text").update_text("Player");
    });

    // Info text
    h.get("content::connection_notice::text").update_on(
        resource_mutation::<MakeLobbyData>(),
        |id: TargetId, mut e: TextEditor, data: ReactRes<MakeLobbyData>| {
            match data.is_single_player() {
                true => write_text!(e, *id, "Single-player lobby: does not require a server connection."),
                false => write_text!(e, *id, "Multiplayer lobby: requires a server connection."),
            };
        },
    );

    // Popup buttons
    h.edit("footer::accept_button", |h| {
        setup_request_tracker::<MakeLobby>(h);

        // This is where the magic happens.
        h.on_pressed(make_a_lobby);

        // Disable button when it can't be used.
        h.enable_if(
            (
                resource_mutation::<ConnectionStatus>(),
                resource_mutation::<MakeLobbyData>(),
                resource_mutation::<LobbyDisplay>(),
                broadcast::<RequestStarted<MakeLobby>>(),
                broadcast::<RequestEnded<MakeLobby>>(),
            ),
            |//
                _: TargetId,
                status: ReactRes<ConnectionStatus>,
                data: ReactRes<MakeLobbyData>,
                make_lobby: PendingRequestParam<MakeLobby>,
                lobby_display: ReactResMut<LobbyDisplay>,//
            | {
                let enable = (*status == ConnectionStatus::Connected) || data.is_single_player();
                // if LobbyDisplay is hosted then we are in a lobby on the host server
                let enable = enable && !make_lobby.has_request() && !lobby_display.is_hosted();
                enable
            },
        );
    });
    // Note: the cancel button doesn't clear the lobby settings in case you want to resume where you left off.
    h.get("footer::cancel_button")
        .on_pressed(move |mut c: Commands| {
            c.get_entity(popup_id)?.despawn();
            DONE
        });
}

//-------------------------------------------------------------------------------------------------------------------

/// Event broadcast to activate the popup.
#[derive(Debug)]
pub(crate) struct ActivateMakeLobbyPopup;

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct UiMakeLobbyPopupPlugin;

impl Plugin for UiMakeLobbyPopupPlugin
{
    fn build(&self, app: &mut App)
    {
        app.add_reactor(
            broadcast::<ActivateMakeLobbyPopup>(),
            setup_broadcast_popup(("user.sections.play", "make_lobby_popup"), build_make_lobby_popup),
        );
    }
}

//-------------------------------------------------------------------------------------------------------------------
