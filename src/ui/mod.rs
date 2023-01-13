use bevy::app::{App, Plugin};

mod start;
mod stats;
mod game;


pub struct UI;

impl Plugin for UI {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(start::UIStartPlugin)
            .add_plugin(game::UIGamePlugin)
            .add_plugin(stats::UIStatsPlugin);

        ;
    }
}


