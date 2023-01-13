use leafwing_input_manager::Actionlike;


#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum GameFlowActions {
    StartGame,
    StartMatch,

}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum CameraActions {
    Left,
    Right,
    Up,
    Down,
    Reset
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum MatchActions {
    ArticulateLeft,
    ArticulateRight,
    SpawnOrLaunchBall
}
