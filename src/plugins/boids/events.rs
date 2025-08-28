use bevy::prelude::*;

#[derive(Event)]
pub struct SpawnBoid {
    pub position: Vec2,
    pub facing: f32,
    pub speed: f32,
}
