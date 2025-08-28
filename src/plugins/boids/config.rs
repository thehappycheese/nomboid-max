use bevy::prelude::*;


#[derive(Resource)]
pub struct BoidConfig {
    pub boid_vision_radius:f32,
    pub boid_crowding_radius:f32,
    pub boid_vision_cone_radius_radians:f32,
    pub boid_base_speed:f32,
    pub average_group_direction:f32,
    pub average_group_position:f32,
    pub avoid_crowding:f32,
    pub target_boid_population:usize,
}

impl Default for BoidConfig {
    fn default() -> Self {
        Self { 
            boid_vision_radius              : 30.0,
            boid_crowding_radius            : 10.0,
            boid_vision_cone_radius_radians : 110f32.to_radians(),
            boid_base_speed                 : 0.6,
            average_group_direction         : 0.02,
            average_group_position          : 0.03,
            avoid_crowding                  : 0.04,
            target_boid_population          : 5000,
        }
    }
}
