use std::collections::{BTreeSet, HashMap};

use bevy::prelude::*;
use bevy_cobweb::prelude::*;
use bevy_cobweb_ui::prelude::*;
use client_core::*;
use game_core::*;

//-------------------------------------------------------------------------------------------------------------------

#[derive(ReactResource, Default)]
struct Scoreboard
{
    entries: HashMap<Entity, PlayerScore>,
    ordered: BTreeSet<(PlayerScore, Entity)>,
}

impl Scoreboard
{
    fn update(&mut self, player: Entity, new_score: PlayerScore)
    {
        let score = self.entries.entry(player).or_default();
        let prev_score = *score;
        *score = new_score;

        self.ordered.remove(&(prev_score, player));
        self.ordered.insert((new_score, player));
    }

    fn get(&self, idx: usize) -> Result<Entity, ()>
    {
        self.ordered
            .iter()
            .nth(self.ordered.len().saturating_sub(idx + 1))
            .map(|(_, e)| *e)
            .ok_or(())
    }

    fn num_players(&self) -> usize
    {
        self.entries.len()
    }
}

//-------------------------------------------------------------------------------------------------------------------

fn refresh_scoreboard(mut c: Commands, players: Query<(Entity, &PlayerScore)>)
{
    let mut scoreboard = Scoreboard::default();
    for (player, score) in players.iter() {
        scoreboard.update(player, *score);
    }
    c.insert_react_resource(scoreboard);
}

//-------------------------------------------------------------------------------------------------------------------

fn get_score_changes(
    mut c: Commands,
    mut scoreboard: ReactResMut<Scoreboard>,
    players: Query<(Entity, &PlayerScore), Changed<PlayerScore>>,
)
{
    if players.is_empty() {
        return;
    }

    let scoreboard = scoreboard.get_mut(&mut c);
    for (player, score) in players.iter() {
        scoreboard.update(player, *score);
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(super) fn edit_scoreboard(mut h: UiSceneHandle)
{
    // Add scoreboard entries.
    h.update_on(
        resource_mutation::<Scoreboard>(),
        |//
            id: TargetId,
            mut num_entries: Local<usize>,
            mut c: Commands,
            mut s: SceneBuilder,
            scoreboard: ReactRes<Scoreboard>,
            //
        |
        {
            let mut builder = c.ui_builder(*id);
            let rank_item = ("ui.skin.game", "scoreboard_rank_item");
            let player_item = ("ui.skin.game", "scoreboard_player_item");
            let score_item = ("ui.skin.game", "scoreboard_score_item");

            while *num_entries < scoreboard.num_players() {
                // Make an entry assigned to index `idx` in the scoreboard.
                // The entry text will update whenever the scoreboard changes.
                // - All entries should update in case the player they are assigned to changes.
                let idx = *num_entries;
                let mut player_text = Entity::PLACEHOLDER;
                let mut score_text = Entity::PLACEHOLDER;
                // These are separate scenes because we are using grid layout.
                builder.spawn_scene(rank_item, &mut s, |h| {
                    h.get("text")
                        .update_text(format!("{}.", idx + 1));
                });
                builder.spawn_scene(player_item, &mut s, |h| {
                    player_text = h.get_entity("text")?;
                    DONE
                });
                builder.spawn_scene(score_item, &mut s, |h| {
                    score_text = h.get_entity("text")?;
                    DONE
                });

                // Entries are never despawned so it's ok to have the reactor on the scoreboard entity.
                builder.update_on(
                    resource_mutation::<Scoreboard>(),
                    move |//
                        _: TargetId,
                        mut e: TextEditor,
                        scoreboard: ReactRes<Scoreboard>,
                        players: Query<(&PlayerName, &PlayerScore)>
                        //
                    |
                    {
                        let (name, score) = players.get(scoreboard.get(idx)?)?;
                        write_text!(e, player_text, "{}", name.name.as_str());
                        write_text!(e, score_text, "{}", score.score());
                        DONE
                    }
                );

                *num_entries += 1;
            }
        },
    );
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(SystemSet, PartialEq, Eq, Debug, Hash, Clone)]
pub(super) struct RefreshScoreboardSet;

//-------------------------------------------------------------------------------------------------------------------

pub(super) struct ScoreboardPlugin;

impl Plugin for ScoreboardPlugin
{
    fn build(&self, app: &mut App)
    {
        app.init_react_resource::<Scoreboard>()
            .add_systems(
                OnEnter(ClientState::Play),
                refresh_scoreboard.in_set(RefreshScoreboardSet),
            )
            .add_systems(
                Update,
                get_score_changes
                    .in_set(ClientLogicSet::Update)
                    .run_if(not(in_state(ClientState::Init))),
            );
    }
}

//-------------------------------------------------------------------------------------------------------------------
