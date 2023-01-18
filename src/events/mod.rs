use bevy::app::{App, Plugin};
use bevy::log::info;
use bevy::prelude::{Commands, EventReader, EventWriter, IntoSystemDescriptor, ResMut, State, SystemSet, Vec3};
use crate::ball::Ball;
use crate::block::{BlockBehaviour, BlockType};
use crate::labels::SystemLabels;
use crate::level::{LevelDefinition, RequestTag};
use crate::player::Player;
use crate::points::{PointsDisplay, PointsDisplayRequest};
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
    BallLost,
    BounceOffPaddle,
    BounceOffWall,
    TargetHit(Vec3, BlockType, BlockBehaviour),
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
    mut player: ResMut<Player>,
    mut level: ResMut<LevelDefinition>,
    mut game_flow: EventWriter<GameFlowEvent>,
) {
    for ev in events.iter() {
        match ev {
            MatchEvent::Start => {
                match_state.reset();
            }

            MatchEvent::BallSpawned => {
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
                player.ball_lost();
                match_state.ball_lost();
                if player.balls_available == 0 && match_state.blocks > 0 {
                    game_flow.send(GameFlowEvent::PlayerLooses);
                }
            }

            MatchEvent::BounceOffPaddle => {
                match_state.add_paddle_bounce();
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

                if match_state.blocks == 0 {
                    game_flow.send(GameFlowEvent::PlayerWins);
                }
            }
        }
    }
}

fn game_flow_handler(
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
                info!("Player wins!");
                let _ = game_state.set(GameState::PostMatchWin);
            }

            GameFlowEvent::PlayerLooses => {
                info!("Player looses!");
                let _ = game_state.set(GameState::PostMatchLoose);
            }

            GameFlowEvent::EndGame => {}
        }
    };
}
