use bevy::prelude::States;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default, States)]
pub enum GameState {
    Start,
    #[default]
    InGame,
    InMatch,
    PostMatch,
    NextLevel,
    MatchResult,

    /// The level editor - see [`crate::editor`].
    Editor,
}
