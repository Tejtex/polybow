use bevy::prelude::*;


pub const ARROW_SPEED: f32 = 300.0;
pub const ARROW_COOLDOWN: f32 = 0.5;
pub const MAX_ARROW_SPEED: f32 = 600.0;

#[derive(Component)]
pub struct Arrow {
    pub damage: f32
}