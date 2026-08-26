use bevy::prelude::SystemSet;

/// Ordering sets for the per-frame simulation systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub enum SystemLabels {
    UpdateWorld,
    UpdateState,
}
