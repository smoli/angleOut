mod layout;

use std::thread::spawn;
use std::time::Duration;
use bevy::app::App;
use bevy::prelude::{AssetServer, Commands, Component, Plugin, Res, Resource, SystemSet, Vec2};
use bevy::utils::default;
use crate::ball::{Ball, ball_spawn};
use crate::block::{Block, BlockBehaviour};
use crate::config::{BLOCK_DEPTH, BLOCK_WIDTH, BLOCK_WIDTH_H};
use crate::level::layout::{generate_block_grid, interpret_grid};
use crate::ship::{Ship, ship_spawn};
use crate::state::GameState;


#[derive(Component)]
pub struct RequestTag;

#[derive(Resource)]
pub struct LevelLayout {
    pub simultaneous_balls: usize,
    pub targets: String,
    pub time_limit: Option<Duration>,
}


impl Default for LevelLayout {
    fn default() -> Self {
        LevelLayout {
            targets: "".to_string(),
            simultaneous_balls: 1,
            time_limit: None,
        }
    }
}


pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                SystemSet::on_enter(GameState::InGame)
                    .with_system(level_spawn)
            )

        ;
    }
}

fn level_spawn(mut commands: Commands) {
    commands
        .spawn(Ball::default())
        .insert(RequestTag)
    ;

    commands
        .spawn(Ship::default())
        .insert(RequestTag);


   /* let positions =  generate_block_grid(5, 10, 3.0);

    for i in 0..positions.len() {
        let pos = positions.get(i).unwrap();

        commands.
        spawn(Block {
            position: pos.clone(),
            behaviour: BlockBehaviour::EvadeUp,
            ..Block::hardling()
        })
            .insert(RequestTag);
    }*/
    let a_level = "AA .. .. .. .. .. .. .. AA
 .. .. .. .. .. .. .. .. ..
 .. .. .. .. CB .. .. .. ..
 .. .. .. .. .. .. .. .. ..
 AA .. .. .. .. .. .. .. AA".to_string();



    if let Some(res) = interpret_grid(a_level, 9, 3.0) {
        for b in res {
        println!("{:?}", b);
            commands
                .spawn(b)
                .insert(RequestTag);
        }
    }

}





