use bevy::app::{App, Plugin};
use bevy::log::info;
use bevy::prelude::{Commands, EventReader, EventWriter, IntoSystemDescriptor, Query, ResMut, State, SystemSet, Vec3};
use crate::ball::Ball;
use crate::block::{BlockBehaviour, BlockType};
use crate::labels::SystemLabels;
use crate::level::{LevelDefinition, RequestTag};
use crate::pickups::{Pickup, PickupType};
use crate::player::{Player, PlayerState};
use crate::points::{PointsDisplay, PointsDisplayRequest};
use crate::powerups::PowerUpType;
use crate::r#match::state::MatchState;
use crate::state::GameState;

#[derive(Debug)]
pub enum GameFlowEvent {
    StartGame,
    StartMatch,

    PlayerWins,
    PlayerLooses,

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
    TargetHit(Vec3, BlockType, BlockBehaviour),
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


fn match_event_handler(
    mut commands: Commands,
    mut events: EventReader<MatchEvent>,
    mut match_state: ResMut<MatchState>,
    mut players: Query<&mut Player>,
    mut level: ResMut<LevelDefinition>,
    mut game_flow: EventWriter<GameFlowEvent>,
) {
    let mut player = players.get_single_mut().unwrap();

    for ev in events.iter() {
        match ev {
            MatchEvent::Start => {
                match_state.reset();
            }

            MatchEvent::BallSpawned => {
                info!("Executing ball spawn request");
                if player.balls_available > 0 && player.balls_spawned == 0 && player.balls_in_play < level.simultaneous_balls {
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
                info!("Ball Lost");
                player.ball_lost();
                match_state.ball_lost();
                if player.balls_available == 0 && match_state.blocks > 0 {
                    game_flow.send(GameFlowEvent::PlayerLooses);
                }
            }

            MatchEvent::BounceOffPaddle => {
                match_state.add_paddle_bounce();
                /*                if let Some(mut bouncer) = player.power_ups.get(&PowerUpType::Bouncer) {
                                    if !bouncer.available() {
                                        game_flow.send(GameFlowEvent::PlayerLooses)
                                    } else {
                                        bouncer.use_one();
                                    }
                                } else {
                                    game_flow.send(GameFlowEvent::PlayerLooses)
                                }
                */
            }

            MatchEvent::BounceOffWall => {
                match_state.add_wall_hit();
            }

            MatchEvent::TargetHit(p, block_type, behaviour) => {
                let (_, awarded) = match_state.add_block_hit(block_type, behaviour);

                commands.spawn(PointsDisplay {
                    text: awarded.to_string(),
                    position: p.clone(),
                }).insert(PointsDisplayRequest);

                /*                commands.spawn(Pickup {
                                    spawn_position: p.clone(),
                                    pickup_type: PickupType::PowerUp(PowerUpType::Grabber { grabs: 5 })
                                }).insert(RequestTag);
                */

                if match_state.blocks == 0 {
                    game_flow.send(GameFlowEvent::PlayerWins);
                }
            }

            MatchEvent::BlockLost => {
                match_state.block_lost();
            }

            MatchEvent::BallGrabbed => {
                player.ball_grabbed();
            }

            MatchEvent::PickedUp(pt) => {
                info!("Player picked up {:?}", pt)
            }
        }
    }
}

fn game_flow_handler(
    mut players: Query<&mut Player>,
    mut events: EventReader<GameFlowEvent>,
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
                    info!("Player wins!");
                    player.state = PlayerState::HasWon;
                    let _ = game_state.set(GameState::PostMatch);
                };
            }

            GameFlowEvent::PlayerLooses => {
                if let Ok(mut player) = players.get_single_mut() {
                    info!("Player looses!");
                    player.state = PlayerState::HasLost;
                    let _ = game_state.set(GameState::PostMatch);
                };
            }

            GameFlowEvent::EndGame => {}
        }
    };
}
