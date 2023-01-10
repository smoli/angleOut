use bevy::app::{App, Plugin};
use bevy::prelude::{EventReader, ResMut, State};
use crate::state::GameState;

#[derive(Debug)]
pub enum GameFlowEvent {
    StartGame,

    EndGame
}

pub enum MatchEvent {
    Start,
    SpawnBall,
    LaunchBall,
    LooseBall,
    BounceOfPaddle,
    DestroyFoe,
    End
}



pub struct EventsPlugin;

impl Plugin for EventsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<MatchEvent>()
            .add_event::<GameFlowEvent>()
            .add_system(game_flow_handler);
    }
}

fn game_flow_handler(
    mut events: EventReader<GameFlowEvent>,
    mut game_state: ResMut<State<GameState>>
) {
    for ev in events.iter() {

        match ev {
            GameFlowEvent::StartGame => {
               let _ = game_state.set(GameState::InGame);
            }
            GameFlowEvent::EndGame => {}
        }
    };
}
