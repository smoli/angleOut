pub mod state;

use bevy::app::App;
use bevy::prelude::{Commands, Component, Entity, EventReader, IntoSystemDescriptor, Plugin, Query, ResMut, SystemSet, With};
use crate::events::MatchEvent;
use crate::labels::SystemLabels;
use crate::state::GameState;

#[derive(Component)]
pub struct Match;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(state::MatchState::default())

            .add_system_set(
                SystemSet::on_enter(GameState::InGame)
                    .with_system(match_spawn.label(SystemLabels::UpdateWorld))
            )

            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .with_system(match_update_state
                        .after(SystemLabels::UpdateWorld)
                        .label(SystemLabels::UpdateState)
                    )
            )

            .add_system_set(
                SystemSet::on_exit(GameState::InGame)
                    .with_system(match_despawn.label(SystemLabels::UpdateWorld))
            );
    }
}


fn match_spawn(mut commands: Commands) {
    commands.spawn(Match);
}

fn match_despawn(mut commands: Commands, matches: Query<Entity, With<Match>>) {
    for the_match in &matches {
        commands.entity(the_match).despawn();
    }
}

fn match_update_state(
    mut events: EventReader<MatchEvent>,
    matches: Query<Entity, With<Match>>,
    mut match_state: ResMut<state::MatchState>,
) {
    for _ in &matches {
        for ev in events.iter() {
            match ev {

                MatchEvent::Start => {
                    match_state.reset();
                }

                MatchEvent::SpawnBall => {
                    // match_state.balls_active += 1;
                    // match_state.balls_available -= 1;
                }
                MatchEvent::LaunchBall => {}
                MatchEvent::LooseBall => {
                    // match_state.balls_active -= 1;
                }

                MatchEvent::BounceOfPaddle => {
                    match_state.paddle_bounces += 1;
                }
                MatchEvent::DestroyFoe => {
                    // match_state.blocks -= 1;
                }

                MatchEvent::End => {}
            }
        }
    }
}