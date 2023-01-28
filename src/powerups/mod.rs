use bevy::prelude::Component;


#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PowerUpType {
    Grabber(u16)
}



#[derive(Component)]
pub struct Bouncer {
    pub bounces: i16
}

#[derive(Component)]
pub struct Grabber {
    pub grabs: i16
}
