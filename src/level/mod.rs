mod layout;

use std::thread::spawn;
use std::time::Duration;
use bevy::app::App;
use bevy::prelude::{AssetServer, Commands, Component, IntoSystemDescriptor, Plugin, Res, ResMut, Resource, SystemSet, Vec2};
use bevy::utils::default;
use crate::ball::{Ball, ball_spawn};
use crate::block::{Block, BlockBehaviour, BlockType};
use crate::config::{BLOCK_DEPTH, BLOCK_GAP, BLOCK_WIDTH, BLOCK_WIDTH_H};
use crate::labels::SystemLabels;
use crate::level::layout::{generate_block_grid, interpret_grid};
use crate::level::TargetLayout::FilledGrid;
use crate::r#match::state::MatchState;
use crate::ship::{Ship, ship_spawn};
use crate::state::GameState;


#[derive(Component)]
pub struct RequestTag;


pub enum TargetLayout {
    FilledGrid(usize, usize, BlockType, BlockBehaviour, f32),
    SparseGrid(String, f32),
}


#[derive(Resource)]
pub struct LevelDefinition {
    pub simultaneous_balls: i32,
    pub targets: TargetLayout,
    pub time_limit: Option<Duration>,
}


impl Default for LevelDefinition {
    fn default() -> Self {
        LevelDefinition {
            targets: FilledGrid(10, 5, BlockType::Simple, BlockBehaviour::SittingDuck, BLOCK_GAP),
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
                SystemSet::on_enter(GameState::InMatch)
                    .with_system(level_spawn.label(SystemLabels::UpdateWorld))
            )

        ;
    }
}


fn make_filled_grid(
    mut commands: &mut Commands,
    cols: usize, rows: usize, block_type: &BlockType, behaviour: &BlockBehaviour, gap: f32) -> i32
{
    let positions = generate_block_grid(rows, cols, gap);

    for i in 0..positions.len() {
        let pos = positions.get(i).unwrap();

        commands.
            spawn(Block {
                position: pos.clone(),
                behaviour: behaviour.clone(),
                block_type: block_type.clone(),
                ..default()
            })
            .insert(RequestTag);
    }

    positions.len() as i32
}

fn make_grid_from_string_layout(
    mut commands: &mut Commands,
    layout: &String,
    gap: f32,
) -> i32 {
    if let Some(res) = interpret_grid(layout, gap) {
        let mut c = 0;
        for b in res {
            println!("{:?}", b);
            commands
                .spawn(b)
                .insert(RequestTag);

            c += 1;
        }

        return c;
    }

    0
}

fn level_spawn(
    mut stats: ResMut<MatchState>,
    level: Res<LevelDefinition>,
    mut commands: Commands) {

    commands
        .spawn(Ship::default())
        .insert(RequestTag);


    let count = match &level.targets {
        FilledGrid(cols, rows, block_type, behaviour, gap) => {
            make_filled_grid(&mut commands, *cols, *rows, block_type, behaviour, *gap)
        }
        TargetLayout::SparseGrid(layout, gap) => {
            make_grid_from_string_layout(&mut commands, layout, *gap)
        }
    };

    stats.set_block_count(count);
}





