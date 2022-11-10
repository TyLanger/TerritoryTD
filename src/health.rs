#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    max_health: u32,
    current_health: u32,
    just_died: bool,
}

impl Health {
    pub fn new(max_health: u32) -> Self {
        assert!(max_health > 0, "Max health needs to be above 0");
        Health {
            max_health,
            current_health: max_health,
            just_died: false,
        }
    }

    pub fn take_damage(&mut self, damage: u32) {
        self.just_died = false;
        if self.is_dead() {
            // already dead
            return;
        }
        if damage >= self.current_health {
            self.current_health = 0;
            // just died
            // for tracking who killed you
            self.just_died = true;
        } else {
            self.current_health -= damage;
        }
    }

    pub fn is_dead(&self) -> bool {
        self.current_health == 0
    }

    /// Died to the last hit.
    /// For tracking who killed this
    pub fn just_died(&self) -> bool {
        self.just_died
    }
}
