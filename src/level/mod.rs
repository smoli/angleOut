use std::time::Duration;

use bevy::app::{App, Plugin};
use bevy::log::{error, warn};
use bevy::math::Vec2;
use bevy::platform::collections::HashMap;
use bevy::prelude::{default, Commands, Component, IntoScheduleConfigs, OnEnter, ResMut, Resource, Vec3};
use rand::{rng, RngExt};
use serde::{Deserialize, Serialize};

use crate::block::{Block, BlockBehaviour, BlockType};
use crate::config::{ARENA_WIDTH_H, BLOCK_GAP, BLOCK_WIDTH};
use crate::labels::SystemLabels;
use crate::level::layout::{generate_block_grid, interpret_grid};
use crate::level::TargetLayout::{FilledGrid, SparseGrid};
use crate::pickups::PickupType;
use crate::r#match::state::MatchState;
use crate::ship::Ship;
use crate::state::GameState;

pub mod campaign;
mod layout;

#[derive(Component)]
pub struct RequestTag;


/// How a level's blocks are laid out.
///
/// `SparseGrid` carries the block map as the multi-line ASCII token string
/// documented in `layout::make_block`, so a level file stays readable and
/// hand-editable; the `f32` is the gap between cells.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TargetLayout {
    FilledGrid(usize, usize, BlockType, BlockBehaviour, f32),
    SparseGrid(String, f32),
    Custom(String)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LevelObstacle {
    // Center position, width, height
    Box(Vec3, f32, f32),

    // Center position, Normal, width, flip normal when rotating in place (hacky)
    ForceField(Vec3, Vec3, f32, bool),

    DirectionalDeathTrigger(Vec3, Vec3, f32)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WinCriteria {
    BlockHitPercentage(f32)
}

/// One level, as it is authored in `assets/levels/*.ron`.
///
/// Every field defaults, so a level file only has to name what it changes from
/// [`LevelDefinition::default`] - the same shape the hardcoded levels had with
/// `..default()`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct LevelDefinition {
    pub background_asset: String,
    pub background_scroll_velocity: f32,
    pub simultaneous_balls: i32,
    pub win_criteria: WinCriteria,
    pub targets: TargetLayout,
    pub time_limit: Option<Duration>,
    pub global_pickups: Vec<PickupType>,

    /// Derived at runtime by [`LevelDefinition::distribute_global_pickups`] from
    /// `global_pickups` and the level's block count - never authored, so it is
    /// kept out of the file.
    #[serde(skip)]
    pub distributed_global_pickups: HashMap<usize, PickupType>,
    pub obstacles: Vec<LevelObstacle>,
    pub default_wall_l: bool,
    pub default_wall_r: bool,

}

impl Default for LevelDefinition {
    fn default() -> Self {
        return LevelDefinition {
            background_asset: "ship3_003.glb#Scene10".to_string(),
            background_scroll_velocity: 0.0,
            simultaneous_balls: 1,
            win_criteria: WinCriteria::BlockHitPercentage(1.0),
            targets: FilledGrid(5, 5, BlockType::Simple, BlockBehaviour::SittingDuck, BLOCK_GAP),
            time_limit: None,
            global_pickups: vec![],
            distributed_global_pickups: Default::default(),
            obstacles: vec![],
            default_wall_l: true,
            default_wall_r: true
        }
    }
}


#[derive(Resource)]
pub struct Levels {
    pub definitions: Vec<LevelDefinition>,
    pub current_level: usize
}


impl Levels {
    pub fn get_current_level(&self) -> Option<&LevelDefinition> {
        self.definitions.get(self.current_level.clone())
    }

    pub fn get_current_level_mut(&mut self) -> Option<&mut LevelDefinition> {
        self.definitions.get_mut(self.current_level.clone())
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


impl LevelDefinition {
    pub fn pickup_at(&self, remaining_block_count: usize) -> Option<&PickupType> {
        self.distributed_global_pickups.get(&remaining_block_count)
    }

    pub fn clear_pickups(&mut self) {
        self.distributed_global_pickups.clear();
    }

    pub fn distribute_global_pickups(&mut self, block_count: usize) {
        let mut rng = rng();
        self.distributed_global_pickups.clear();


        let mut placed: Vec<usize> = vec![];
        let mut start = 0;
        let mut end = block_count;
        //info!("Distributing {} global pickups", self.global_pickups.len());
        for pickup in &self.global_pickups {
            let bc = block_count.clone();
            let mut repeats = 10;

            while repeats > 0 {
                repeats -= 1;
                let pos: usize = rng.random_range(start..end);

                if placed.contains(&pos) {
                    warn!("Moving start and end around should avoid this!");
                } else {
                    if pos == bc - 1 {
                        end = pos + 1;
                    } else {
                        start = pos;
                    }
                    placed.push(pos.clone());
                    self.distributed_global_pickups.insert(pos.clone(), pickup.clone());

                    //info!("{:?} at {}", pickup, pos);
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
            .add_systems(
                OnEnter(GameState::InMatch),
                level_spawn.in_set(SystemLabels::UpdateWorld),
            )

        ;
    }
}


fn make_filled_grid(
    commands: &mut Commands,
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
    commands: &mut Commands,
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


    let level = levels.get_current_level_mut().unwrap();

    match &level.targets {
        FilledGrid(cols, rows, block_type, behaviour, gap) => {
            let count = make_filled_grid(&mut commands, cols.clone(), rows.clone(), block_type, behaviour, gap.clone());
            level.distribute_global_pickups(count as usize);
            stats.set_block_count(count.clone());

        }

        SparseGrid(layout, gap) => {
            let count = make_grid_from_string_layout(&mut commands, layout, gap.clone());
            level.distribute_global_pickups(count as usize);
            stats.set_block_count(count.clone());
        }

        TargetLayout::Custom(name) => {

            match name.as_str() {
                "Conveyor" => {
                    level_span_conveyor(stats, level, commands);
                }

                _ => {
                    error!("Unknown custom level definition {}", name);
                }
            };

        }
    };

}

fn level_span_conveyor(
    mut stats: ResMut<MatchState>,
    level: &mut LevelDefinition,
    mut commands: Commands
) {
    let count_per_row = 2;
    let mut pos = Vec2::new(ARENA_WIDTH_H + 3.0, -25.0);
    for _i in 0..count_per_row {
        let speed = 10.0;
        commands.
            spawn(Block {
                position: pos.clone(),
                behaviour: BlockBehaviour::EvaderL(speed),
                block_type: BlockType::Simple,
                ..default()
            })
            .insert(RequestTag);
        pos.x += 2.0 * BLOCK_WIDTH + BLOCK_GAP;
    }


    let mut pos = Vec2::new(-ARENA_WIDTH_H - 3.0, -35.0);
    for _i in 0..count_per_row {
        let speed2 = 10.0;
        commands.
            spawn(Block {
                position: pos.clone(),
                behaviour: BlockBehaviour::EvaderR(speed2),
                block_type: BlockType::Simple,
                ..default()
            })
            .insert(RequestTag);
        pos.x -= 2.0 * BLOCK_WIDTH + BLOCK_GAP;
    }

    level.clear_pickups();
    stats.set_block_count(2 * count_per_row);
}





