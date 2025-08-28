use bevy::prelude::*;

#[derive(Component)]
pub struct Boid {
    pub facing:f32,
    pub speed:f32,
}

#[derive(Component)]
pub struct BoidZone {
    pub width:f32,
    pub height:f32
}

#[derive(Component, Default)]
pub struct TrackedByKDTree;
