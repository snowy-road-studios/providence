use std::vec::Vec;

use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_girk_backend_public::*;
use wiring_backend::*;

//-------------------------------------------------------------------------------------------------------------------

/// Caches the currently-displayed lobby page.
///
/// This is a reactive resource.
#[derive(ReactResource, Debug, Default)]
pub(crate) struct LobbyPage
{
    /// Current lobby contents.
    current: Vec<ProvLobbyContents>,
    /// Number of lobbies younger than the current page on the server.
    num_younger: usize,
    /// Total number of lobbies on the server.
    total: usize,
}

impl LobbyPage
{
    pub(crate) fn try_set(&mut self, new_page: LobbySearchResult) -> Result<(), String>
    {
        // extract lobby contents
        let mut temp = Vec::with_capacity(new_page.lobbies.len());

        for lobby_data in new_page.lobbies {
            temp.push(ProvLobbyContents::try_from(lobby_data)?);
        }

        // update the page
        self.current = temp;
        self.num_younger = new_page.num_younger;
        self.total = new_page.total;

        Ok(())
    }

    pub(crate) fn _clear(&mut self)
    {
        self.current = vec![];
    }

    pub(crate) fn get(&self) -> &Vec<ProvLobbyContents>
    {
        &self.current
    }

    pub(crate) fn len(&self) -> usize
    {
        self.current.len()
    }

    /// Returns (start count, end count, total lobbies)
    pub(crate) fn stats(&self) -> (usize, usize, usize)
    {
        (self.num_younger + 1, self.num_younger + self.current.len(), self.total)
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Tracks the last lobby search request sent to the host server.
///
/// On startup this is initialized with the top-most lobby page.
#[derive(ReactResource, Debug)]
pub(crate) struct LobbyPageRequest
{
    last: LobbySearchRequest,
}

impl LobbyPageRequest
{
    pub(crate) fn new(request: LobbySearchRequest) -> Self
    {
        Self { last: request }
    }

    pub(crate) fn set(&mut self, request: LobbySearchRequest)
    {
        self.last = request;
    }

    pub(crate) fn get(&self) -> &LobbySearchRequest
    {
        &self.last
    }

    pub(crate) fn is_now(&self) -> bool
    {
        match self.last {
            LobbySearchRequest::PageOlder { youngest_id, num: _ } => youngest_id == u64::MAX,
            _ => false,
        }
    }

    pub(crate) fn is_oldest(&self) -> bool
    {
        match self.last {
            LobbySearchRequest::PageNewer { oldest_id, num: _ } => oldest_id == 0u64,
            _ => false,
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LobbyPagePlugin;

impl Plugin for LobbyPagePlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_react_resource::<LobbyPage>()
            .insert_react_resource(LobbyPageRequest::new(LobbySearchRequest::PageOlder {
                youngest_id: u64::MAX,
                num: LOBBY_LIST_SIZE as u16,
            }));
    }
}

//-------------------------------------------------------------------------------------------------------------------
