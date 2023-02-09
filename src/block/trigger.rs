use bevy::prelude::{Bundle, Component, Resource};
use bevy::utils::HashMap;
use crate::block::trigger::TriggerState::{Started, Stopped};


pub type TriggerGroup = u16;

#[derive(PartialEq, Debug, Clone)]
pub enum TriggerType {
    Start,
    Stop,
    StartStop,
    ReceiverStartingInactive,
    ReceiverStartingActive
}

#[derive(Component, Debug)]
pub struct BlockTrigger {
    pub group: TriggerGroup,
    pub trigger_type: TriggerType
}

#[derive(Component)]
pub struct BlockTriggerTargetInactive;

#[derive(Component, Debug)]
pub struct BlockTriggerTarget {
    pub group: TriggerGroup
}


#[derive(PartialEq)]
pub enum TriggerState {
    Started,
    Stopped
}

#[derive(Resource)]
pub struct TriggerStates {
    states: HashMap<TriggerGroup, TriggerState>,
    consumed: HashMap<TriggerGroup, bool>
}

impl TriggerStates {

    pub fn new() -> Self {
        return TriggerStates {
            states: HashMap::new(),
            consumed: HashMap::new()
        }
    }

    pub fn is_consumed(&self, group: TriggerGroup) -> bool {
        match self.consumed.get(&group) {
            None => false,
            Some(c) => *c
        }
    }

    pub fn consume(&mut self, group: TriggerGroup) {
        self.consumed.insert(group, true);
    }

    fn unconsume(&mut self, group: TriggerGroup) {
        self.consumed.insert(group, false);
    }

    pub fn start(&mut self, group: TriggerGroup) {
        self.states
            .entry(group)
            .and_modify(|e| *e = Started)
            .or_insert(Started)
        ;
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

    pub fn flip(&mut self, group: TriggerGroup) {
        if self.is_stopped(group) {
            self.start(group);
        } else {
            self.stop(group);
        }
    }

    pub fn is_started(&self, group: TriggerGroup) -> bool {
        match self.states.get(&group) {
            None => false,
            Some(s) => *s == Started
        }
    }

    pub fn is_stopped(&self, group: TriggerGroup) -> bool {
        match self.states.get(&group) {
            None => true,
            Some(s) => *s == Stopped
        }
    }
}


