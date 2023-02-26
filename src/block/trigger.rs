use bevy::prelude::{Bundle, Component, Resource};
use bevy::utils::HashMap;
use crate::block::trigger::TriggerState::{Started, Stopped};
use crate::physics::Collision;


pub type TriggerGroup = u16;

#[derive(PartialEq, Debug, Clone)]
pub enum TriggerType {
    Start,
    Stop,
    StartStop,
    ReceiverStartingInactive,
    ReceiverStartingActive,
}

#[derive(Component, Debug)]
pub struct BlockTrigger {
    pub group: TriggerGroup,
    pub trigger_type: TriggerType,
}

#[derive(Component)]
pub struct BlockTriggerTargetInactive;

#[derive(Component, Debug)]
pub struct BlockTriggerTarget {
    pub group: TriggerGroup,
    pub auto_stop: bool,
}

#[derive(PartialEq)]
pub enum TriggerState {
    Started(Collision),
    Stopped,
}

#[derive(Resource)]
pub struct TriggerStates {
    states: HashMap<TriggerGroup, TriggerState>,
    consumed: HashMap<TriggerGroup, bool>,
}

impl TriggerStates {
    pub fn new() -> Self {
        return TriggerStates {
            states: HashMap::new(),
            consumed: HashMap::new(),
        };
    }

    pub fn is_consumed(&self, group: TriggerGroup) -> bool {
        match self.consumed.get(&group) {
            None => false,
            Some(c) => *c
        }
    }

    pub fn get_state(&self, group: TriggerGroup) -> Option<&TriggerState> {
        self.states.get(&group)
    }

    pub fn consume(&mut self, group: TriggerGroup) {
        self.consumed.insert(group, true);
    }

    fn unconsume(&mut self, group: TriggerGroup) {
        self.consumed.insert(group, false);
    }

    pub fn start(&mut self, group: TriggerGroup, collision: Collision) {
        match self.states.get(&group) {
            None => {
                self.states.insert(group, Started(collision));
            },
            Some(_) => {
                self.states
                    .entry(group)
                    .and_modify(|e| *e = Started(collision));
            }
        }
        self.unconsume(group);
    }


    pub fn stop(&mut self, group: TriggerGroup) {
        self.states
            .entry(group)
            .and_modify(|e| *e = Stopped)
            .or_insert(Stopped)
        ;
        self.unconsume(group);
    }

    pub fn flip(&mut self, group: TriggerGroup, collision: Collision) {
        if self.is_stopped(group) {
            self.start(group, collision);
        } else {
            self.stop(group);
        }
    }

    pub fn is_started(&self, group: TriggerGroup) -> bool {
        match self.states.get(&group) {
            None => false,
            Some(s) => if let Started(_) =*s  { true } else { false }
        }
    }

    pub fn is_stopped(&self, group: TriggerGroup) -> bool {
        match self.states.get(&group) {
            None => true,
            Some(s) => *s == Stopped
        }
    }
}


