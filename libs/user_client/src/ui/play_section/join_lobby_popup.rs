use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;

use crate::*;

//-------------------------------------------------------------------------------------------------------------------

fn update_join_lobby_data(
    event: BroadcastEvent<ActivateJoinLobbyPopup>,
    mut c: Commands,
    lobby_page: ReactRes<LobbyPage>,
    mut data: ReactResMut<JoinLobbyData>,
) -> DropErr
{
    // lobby id of lobby to join
    let event = event.try_read()?;
    let lobby_index = event.lobby_list_index;

    let Some(lobby_contents) = lobby_page.get().get(lobby_index) else {
        tracing::error!("failed accessing lobby contents for join lobby popup; index={lobby_index}");
        return DONE;
    };

    // update the cached lobby contents
    *data.get_mut(&mut c) = JoinLobbyData { contents: Some(lobby_contents.clone()), ..Default::default() };

    DONE
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn build_join_lobby_popup(_: &ActivateJoinLobbyPopup, h: &mut UiSceneHandle)
{
    // Reactors for auto-closing the popup.
    h.reactor(
        broadcast::<RequestEnded<JoinLobby>>(),
        |//
            id: TargetId,
            event: BroadcastEvent<RequestEnded<JoinLobby>>,
            mut c: Commands//
        |
        {
            match event.try_read()? {
                RequestEnded::Success => {
                    tracing::info!("JoinLobby request succeeded");
                    c.get_entity(*id)?.despawn();
                }
                RequestEnded::Failure => {
                    tracing::warn!("JoinLobby request failed");
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

    // Sub-title
    h.get("subtitle::text").update_on(
        resource_mutation::<JoinLobbyData>(),
        |id: TargetId, mut e: TextEditor, data: ReactRes<JoinLobbyData>| {
            let contents = data.contents.as_ref().result()?;
            let lobby_id = contents.id % 1_000_000u64;
            let owner_id = contents.owner_id % 1_000_000u128;
            write_text!(e, *id, "Lobby: {:0>6} -- Owner: {:0>6}", lobby_id, owner_id);
            OK
        },
    );

    // Form fields
    h.edit("content::grid::password_field", |_| {
        // does nothing yet
    });
    h.edit("content::grid::join_as_field", |h| {
        h.get("text").update_text("Player");
    });

    // Popup buttons
    h.edit("footer::accept_button", |h| {
        setup_request_tracker::<JoinLobby>(h);

        // This is where the magic happens.
        h.on_pressed(send_join_lobby_request);

        // Disable button when it can't be used.
        h.enable_if(
            (
                resource_mutation::<ConnectionStatus>(),
                broadcast::<RequestStarted<JoinLobby>>(),
                broadcast::<RequestEnded<JoinLobby>>(),
            ),
            |_: TargetId, status: ReactRes<ConnectionStatus>, join_lobby: PendingRequestParam<JoinLobby>| {
                let enable = *status == ConnectionStatus::Connected;
                let enable = enable && !join_lobby.has_request();
                enable
            },
        );
    });
    h.get("footer::cancel_button")
        .on_pressed(move |mut c: Commands, mut data: ReactResMut<JoinLobbyData>| {
            c.get_entity(popup_id)?.despawn();
            data.get_mut(&mut c).clear();
            DONE
        });
}

//-------------------------------------------------------------------------------------------------------------------

/// This is a reactive data event used to activate the window.
#[derive(Debug, Clone)]
pub(crate) struct ActivateJoinLobbyPopup
{
    /// Index in the lobby list of the lobby corresponding to this lobby window.
    ///
    /// Only valid in the tick where it is set.
    pub(crate) lobby_list_index: usize,
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct UiJoinLobbyPopupPlugin;

impl Plugin for UiJoinLobbyPopupPlugin
{
    fn build(&self, app: &mut App)
    {
        app
            // must order this before the popup constructor so popup data is correct
            .add_reactor(broadcast::<ActivateJoinLobbyPopup>(), update_join_lobby_data)
            .add_reactor(
                broadcast::<ActivateJoinLobbyPopup>(),
                setup_broadcast_popup(("user.sections.play", "join_lobby_popup"), build_join_lobby_popup),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------
