use bevy::app::{App, Plugin};
use bevy::prelude::{ResMut, SystemSet};
use crate::player::Player;
use crate::powerups::PowerUpType;
use crate::state::GameState;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Player::default())
            .add_system_set(
                SystemSet::on_enter(GameState::InMatch)
                    .with_system(game_start)
            )

        ;
    }
}

fn game_start(
    mut player:ResMut<Player>
) {
    player.reset();
    player.set_balls(3);
    player.power_ups.push(PowerUpType::Grabber(5));
}

