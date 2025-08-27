use bevy::prelude::*;

// use crate::{components::proximity_grid::ProximityGridStatus, Boid, BoidProximityGrid, BOID_RADIUS};

// pub fn grid_neighbors(
//    boids: Query<(Entity, &Transform, &ProximityGridStatus), With<Boid>>,
//    grid: Res<BoidProximityGrid>,
//    mut gizmos: Gizmos,
// ) {
//    for (entity, transform, proximity_status) in boids.iter() {
//        let Some((cell_x, cell_y)) = proximity_status.inserted_at else { continue };
//        gizmos.circle_2d(transform.translation.truncate(), BOID_RADIUS, Color::BLACK);

//        for neighbor_entity in grid.get_neighbors_from_cell_coordinates(cell_x, cell_y) {
//            if entity == neighbor_entity { continue }
           
//            if let Ok((_, neighbor_transform, _)) = boids.get(neighbor_entity) {
//                gizmos.line_2d(
//                    transform.translation.truncate(),
//                    neighbor_transform.translation.truncate(),
//                    Color::WHITE
//                );
//            }
//        }
//    }
// }