#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum GameState {
    Start,
    InGame,
    InMatch,
    PostMatch,
    MatchResult
}