use std::thread::spawn;
use std::time::Duration;
use bevy::app::App;
use bevy::prelude::{AssetServer, Commands, Component, Plugin, Res, Resource, SystemSet, Vec2};
use bevy::utils::default;
use crate::ball::{Ball, ball_spawn};
use crate::block::Block;
use crate::config::{BLOCK_DEPTH, BLOCK_WIDTH, BLOCK_WIDTH_H};
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


fn generate_block_grid(
    rows: u32,
    cols: u32,
    gap: f32,
)   -> Vec<Vec2>

{
    let mut y = -3.0;

    let x_step = BLOCK_WIDTH + gap;
    let cols_h = (cols / 2) as f32;

    let mut res = vec![];

    for _ in 0..rows {
        let mut x = 0.0;
        if cols % 2 == 1 {
            x -= cols_h * x_step;
        } else {
            x -= cols_h * x_step - gap / 2.0 - BLOCK_WIDTH_H;
        }

        for _ in 0..cols {
            res.push(Vec2::new(x, y));
            x += x_step;
        }

        y -= BLOCK_DEPTH * 2.0 - gap;
    };

    res
}


fn level_spawn(mut commands: Commands) {
    commands
        .spawn(Ball::default())
        .insert(RequestTag)
    ;

    commands
        .spawn(Ship::default())
        .insert(RequestTag);


    let positions = generate_block_grid(5, 10, 0.3);

    for i in 0..positions.len() {
        let pos = positions.get(i).unwrap();

        commands.
        spawn(Block {
            position: pos.clone(),
            ..default()
        })
            .insert(RequestTag);
    }
}





