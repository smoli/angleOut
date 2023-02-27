use std::os::macos::raw::stat;
use bevy::app::{App, Plugin};
use bevy::log::info;
use bevy::prelude::{Commands, Entity, EventReader, EventWriter, IntoSystemDescriptor, Query, ResMut, State, SystemSet, Vec3};
use crate::ball::Ball;
use crate::block::{BlockBehaviour, BlockType};
use crate::labels::SystemLabels;
use crate::level::{LevelDefinition, Levels, RequestTag, WinCriteria};
use crate::pickups::{Pickup, PickupType};
use crate::player::{Player, PlayerState};
use crate::points::{PointsDisplay, PointsDisplayRequest};
use crate::powerups::{Bouncer, Grabber, PowerUpData, PowerUpType};
use crate::r#match::state::MatchState;
use crate::state::GameState;

#[derive(Debug)]
pub enum GameFlowEvent {
    StartGame,
    StartMatch,

    PlayerWins,
    PlayerLooses,

    NextLevel,

    EndGame,
}

pub enum MatchEvent {
    Start,
    BallSpawned,
    BallLaunched,
    BallGrabbed,
    BallLost,
    BlockLost,
    BounceOffPaddle,
    BounceOffWall,
    BlockHit(Vec3, BlockType, BlockBehaviour),
    PickedUp(PickupType),
}


pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<MatchEvent>()
            .add_event::<GameFlowEvent>()
            .add_system(game_flow_handler)
            .add_system_set(
                SystemSet::on_update(GameState::InMatch)
                    .with_system(match_event_handler
                        .after(SystemLabels::UpdateWorld)
                        .label(SystemLabels::UpdateState))
            )

        ;
    }
}


#[derive(PartialEq)]
enum LevelEndState {
    Won,
    Lost,
    Undecided,
}


fn check_win_criteria(
    win_criteria: &WinCriteria,
    player: &Player,
    stats: &MatchState,
) -> LevelEndState {

    if stats.blocks == 0 {
        match win_criteria {
            WinCriteria::BlockHitPercentage(pct) => {

                let result = (stats.blocks_hit as f32) / (stats.blocks_hit as f32 + stats.blocks_lost as f32);
                return if result >= *pct {
                    LevelEndState::Won
                } else {
                    LevelEndState::Lost
                }
            }
        };
    }

    if player.balls_available == 0 && stats.blocks > 0 && player.balls_in_play == 0 {
        return LevelEndState::Lost;
    }

    LevelEndState::Undecided
}

fn match_event_handler(
    mut commands: Commands,
    mut events: EventReader<MatchEvent>,
    mut match_state: ResMut<MatchState>,
    mut players: Query<(Entity, &mut Player, &mut Bouncer)>,
    mut levels: ResMut<Levels>,
    mut game_flow: EventWriter<GameFlowEvent>,
) {
    let (player_entity, mut player, mut bouncer) = players.get_single_mut().unwrap();

    let mut level = levels.get_current_level().unwrap();

    for ev in events.iter() {
        match ev {
            MatchEvent::Start => {
                match_state.reset();
            }

            MatchEvent::BallSpawned => {
                //info!("Executing ball spawn request");
                if player.balls_available > 0 && player.balls_carried == 0 && player.balls_in_play < level.simultaneous_balls {
                    commands
                        .spawn(Ball::default())
                        .insert(RequestTag)
                    ;
                    player.ball_spawned();
                }
            }

            MatchEvent::BallLaunched => {
                player.ball_launched();
            }

            MatchEvent::BallLost => {
                //info!("Ball Lost");
                player.ball_lost();
                match_state.ball_lost();
            }

            MatchEvent::BounceOffPaddle => {
                match_state.add_paddle_bounce();
                if !bouncer.available() {
                    game_flow.send(GameFlowEvent::PlayerLooses)
                } else {
                    bouncer.use_one();
                }
            }

            MatchEvent::BounceOffWall => {
                match_state.add_wall_hit();
            }

            MatchEvent::BlockHit(p, block_type, behaviour) => {
                let (_, awarded) = match_state.add_block_hit(block_type, behaviour);

                commands.spawn(PointsDisplay {
                    text: awarded.to_string(),
                    position: p.clone(),
                }).insert(PointsDisplayRequest);
            }

            MatchEvent::BlockLost => {
                match_state.block_lost();
            }

            MatchEvent::BallGrabbed => {
                player.ball_grabbed();
            }

            MatchEvent::PickedUp(pt) => {
                commands.entity(player_entity)
                    .insert(Pickup {
                        spawn_position: Default::default(),
                        pickup_type: *pt,
                    });

                //info!("Player picked up {:?}", pt)
            }
        }

        match check_win_criteria(&level.win_criteria, &player, &match_state) {
            LevelEndState::Won => game_flow.send(GameFlowEvent::PlayerWins),
            LevelEndState::Lost => game_flow.send(GameFlowEvent::PlayerLooses),
            LevelEndState::Undecided => {}
        }
    }
}

fn game_flow_handler(
    mut players: Query<&mut Player>,
    mut events: EventReader<GameFlowEvent>,
    mut match_state: ResMut<MatchState>,
    mut game_state: ResMut<State<GameState>>,
) {
    for ev in events.iter() {
        match ev {
            GameFlowEvent::StartGame => {
                let _ = game_state.set(GameState::InGame);
            }

            GameFlowEvent::StartMatch => {
                let _ = game_state.set(GameState::InMatch);
            }

            GameFlowEvent::PlayerWins => {
                if let Ok(mut player) = players.get_single_mut() {
                    //info!("Player wins!");
                    player.state = PlayerState::HasWon;
                    player.player_has_won(match_state.points);
                    //info!("Player now has {} points", player.points);
                    let _ = game_state.set(GameState::PostMatch);
                };
            }

            GameFlowEvent::NextLevel => {
                let _ = game_state.set(GameState::NextLevel);
            }

            GameFlowEvent::PlayerLooses => {
                if let Ok(mut player) = players.get_single_mut() {
                    //info!("Player looses!");
                    player.state = PlayerState::HasLost;
                    let _ = game_state.set(GameState::PostMatch);
                };
            }

            GameFlowEvent::EndGame => {}
        }
    };
}


#[cfg(test)]
mod tests {
    use super::check_win_criteria;

    use bevy::utils::default;
    use crate::level::WinCriteria;
    use crate::player::{Player, PlayerState};
    use crate::r#match::state::MatchState;

    #[test]
    fn just_loosing_a_ball() {
        let stats = MatchState {
            blocks: 1,
            blocks_hit: 1,
            blocks_lost: 1,
            ..default()
        };

        let player = Player {
            balls_available: 1,
            balls_lost: 1,
            ..default()
        };

        let crit = WinCriteria::BlockHitPercentage(1.0);

        assert_eq!(check_win_criteria(&crit, &player, &stats), false);
    }
}