mod components;
pub use components::{Boid, BoidZone};

mod systems;

mod events;
use events::SpawnBoid;

mod config;
pub use config::BoidConfig;

use bevy::prelude::*;

use bevy_spatial::{AutomaticUpdate, SpatialStructure, TransformMode};

pub struct BoidPlugin;
impl Plugin for BoidPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(
            AutomaticUpdate::<Boid>::new()
                .with_spatial_ds(SpatialStructure::KDTree2)
                .with_frequency(std::time::Duration::from_secs_f32(0.3))
                .with_transform(TransformMode::GlobalTransform),
        )
        .insert_resource(config::BoidConfig::default())
        .add_event::<SpawnBoid>()
        .add_systems(
            FixedUpdate,
            (
                systems::boid_rotate_to_face_group,
                systems::boid_move_forward,
                systems::boid_screen_wrap,
                systems::maintain_boid_population,
                systems::spawn_boid,
            ),
        );
    }
}
