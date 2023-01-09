use bevy::prelude::SystemLabel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(SystemLabel)]
pub enum SystemLabels {
    UpdateWorld,
    UpdateState
}