use bevy::prelude::Component;


#[derive(PartialEq, Debug, Clone, Copy, Eq, Hash)]
pub enum PowerUpType {
    Grabber,
    Bouncer
}


pub trait PowerUpData {
    fn available(&self) -> bool;

    fn use_one(&mut self);
}

#[derive(Component)]
pub struct Bouncer {
    pub bounces: i16
}

impl PowerUpData for Bouncer {
    fn available(&self) -> bool {
        self.bounces > 0 || self.bounces < 0
    }

    fn use_one(&mut self) {
        self.bounces -= 1;
    }
}


#[derive(Component)]
pub struct Grabber {
    pub grabs: i16
}


impl PowerUpData for Grabber {
    fn available(&self) -> bool {
        self.grabs > 0
    }

    fn use_one(&mut self) {
        self.grabs -= 1;
    }
}