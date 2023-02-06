use std::time::Duration;

use bevy::app::App;
use bevy::log::{error, info, warn};
use bevy::prelude::{Commands, Component, IntoSystemDescriptor, Plugin, ResMut, Resource, SystemSet};
use bevy::utils::{default, HashMap};
use rand::{Rng, thread_rng};

use crate::block::{Block, BlockBehaviour, BlockType};
use crate::config::BLOCK_GAP;
use crate::labels::SystemLabels;
use crate::level::layout::{generate_block_grid, interpret_grid};
use crate::level::TargetLayout::FilledGrid;
use crate::pickups::PickupType;
use crate::r#match::state::MatchState;
use crate::ship::Ship;
use crate::state::GameState;

mod layout;

#[derive(Component)]
pub struct RequestTag;


pub enum TargetLayout {
    FilledGrid(usize, usize, BlockType, BlockBehaviour, f32),
    SparseGrid(String, f32),
}


pub struct LevelDefinition {
    pub simultaneous_balls: i32,
    pub targets: TargetLayout,
    pub time_limit: Option<Duration>,
    pub global_pickups: Vec<PickupType>,
    pub distributed_global_pickups: HashMap<usize, PickupType>,
}


#[derive(Resource)]
pub struct Levels {
    pub definitions: Vec<LevelDefinition>,
    pub current_level: usize
}


impl Levels {
    pub fn get_current_level(&self) -> Option<&LevelDefinition> {
        self.definitions.get(self.current_level)
    }

    pub fn get_current_level_mut(&mut self) -> Option<&mut LevelDefinition> {
        self.definitions.get_mut(self.current_level)
    }

    pub fn next_level(&mut self) -> bool {
        if self.current_level < self.definitions.len() - 1 {
            self.current_level += 1;
            true
        } else {
            false
        }

    }
}

impl Default for LevelDefinition {
    fn default() -> Self {
        LevelDefinition {
            targets: FilledGrid(10, 5, BlockType::Simple, BlockBehaviour::SittingDuck, BLOCK_GAP),
            simultaneous_balls: 1,
            time_limit: None,
            global_pickups: Vec::new(),
            distributed_global_pickups: HashMap::new(),
        }
    }
}


impl LevelDefinition {
    pub fn pickup_at(&self, remaining_block_count: usize) -> Option<&PickupType> {
        self.distributed_global_pickups.get(&remaining_block_count)
    }

    pub fn distribute_global_pickups(&mut self, block_count: usize) {
        let mut rng = thread_rng();
        self.distributed_global_pickups.clear();


        let mut placed: Vec<usize> = vec![];
        let mut start = 0;
        let mut end = block_count;
        info!("Distributing {} global pickups", self.global_pickups.len());
        for pickup in &self.global_pickups {
            let mut repeats = 10;

            while repeats > 0 {
                repeats -= 1;
                let pos: usize = rng.gen_range(start..end);

                if placed.contains(&pos) {
                    warn!("Moving start and end around should avoid this!");
                } else {
                    if pos == block_count - 1 {
                        end = pos + 1;
                    } else {
                        start = pos;
                    }
                    placed.push(pos);
                    self.distributed_global_pickups.insert(pos, pickup.clone());

                    info!("{:?} at {}", pickup, pos);
                    break;
                }
            }

            if repeats == 0 {
                warn!("Could not distribute all pickups")
            }
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
            if b.block_type != BlockType::Obstacle {
                c += 1;
            }

            commands
                .spawn(b)
                .insert(RequestTag);

        }

        return c;
    }

    0
}

fn level_spawn(
    mut stats: ResMut<MatchState>,
    mut levels: ResMut<Levels>,
    mut commands: Commands) {
    commands
        .spawn(Ship::default())
        .insert(RequestTag);


    let mut level = levels.get_current_level_mut().unwrap();

    let count = match &level.targets {
        FilledGrid(cols, rows, block_type, behaviour, gap) => {
            make_filled_grid(&mut commands, *cols, *rows, block_type, behaviour, *gap)
        }
        TargetLayout::SparseGrid(layout, gap) => {
            make_grid_from_string_layout(&mut commands, layout, *gap)
        }
    };

    level.distribute_global_pickups(count as usize);

    stats.set_block_count(count);
}





