use bevy::prelude::Component;


#[derive(Component)]
pub struct Bouncer {
    pub bounces: i16
}

#[derive(Component)]
pub struct Grabber {
    pub grabs: i16
}