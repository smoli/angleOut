pub mod state;

use bevy::app::App;
use bevy::prelude::{Commands, Component, Entity, EventReader, IntoSystemDescriptor, Plugin, Query, ResMut, SystemSet, With};
use crate::events::MatchEvent;
use crate::labels::SystemLabels;
use crate::r#match::state::MatchState;
use crate::state::GameState;

#[derive(Component)]
pub struct Match;

pub struct MatchPlugin;

impl Plugin for MatchPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(state::MatchState::default())

            .add_system_set(
                SystemSet::on_enter(GameState::InMatch)
                    .with_system(match_spawn.before(SystemLabels::UpdateWorld))
            )

            .add_system_set(
                SystemSet::on_exit(GameState::InMatch)
                    .with_system(match_despawn)
            );
    }
}


fn match_spawn(
    mut match_state: ResMut<MatchState>,
    mut commands: Commands
) {
    match_state.reset();
    commands.spawn(Match);
}

fn match_despawn(mut commands: Commands, matches: Query<Entity, With<Match>>) {
    for the_match in &matches {
        commands.entity(the_match).despawn();
    }
}
