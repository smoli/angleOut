use bevy::prelude::Reflect;
use leafwing_input_manager::Actionlike;


#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum GameFlowActions {
    StartGame,
    StartMatch,
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum CameraActions {
    Left,
    Right,
    Up,
    Down,
    Reset
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
pub enum MatchActions {
    #[actionlike(DualAxis)]
    ArticulateLeft,
    #[actionlike(DualAxis)]
    ArticulateRight,
    ArticulateUp,
    ArticulateDown,
    SpawnOrLaunchBall,
    GrabTheBall
}
