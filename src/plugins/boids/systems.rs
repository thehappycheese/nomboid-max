use bevy_spatial::{kdtree::KDTree2, SpatialAccess};
use rand::Rng;
use bevy::prelude::*;

use crate::util::lerp_angle;
use super::{Boid, BoidConfig, SpawnBoid, BoidZone};


pub fn spawn_boid(
    mut commands     : Commands,
        asset_server : Res<AssetServer>,
    mut spawn_events : EventReader<SpawnBoid>,
){
    let fish = asset_server.load("fish.png");
    for event in spawn_events.read() {
        commands.spawn((
            Boid{facing:event.facing, speed:event.speed},
            Sprite::from_image(fish.clone()),
            Transform::from_xyz(event.position.x, event.position.y,0.0).with_scale((0.1,0.1,0.1).into()),
        ));
    }
}


pub fn maintain_boid_population(
    mut commands  : Commands,
        boids     : Query<Entity, With<Boid>>,
        config    : Res<BoidConfig>,
    mut event     : EventWriter<SpawnBoid>,
        boid_zone : Query<(&BoidZone, &Transform)>,
) {
    let mut rng = rand::rng();
    let num_boids = boids.iter().count();
    if num_boids < config.target_boid_population {
        let Ok((boid_zone_size, boid_zone_transform))  = boid_zone.single() else{return};
        let num_to_spawn = ((config.target_boid_population - num_boids) as f32 / 10.0).ceil() as usize;
        for _ in 0..num_to_spawn {
            event.write(SpawnBoid{
                facing:(rng.random::<f32>()*2.0-1.0)*std::f32::consts::PI,
                speed:rng.random::<f32>()*0.6,
                position:Vec2 { 
                    x: (rng.random::<f32>()*1.0-0.5) * boid_zone_size.width  + boid_zone_transform.translation.x,
                    y: (rng.random::<f32>()*1.0-0.5) * boid_zone_size.height + boid_zone_transform.translation.y,
                }
            });
        }
    }else if num_boids> config.target_boid_population {
        let num_to_despawn = ((num_boids - config.target_boid_population) as f32 / 10.0).ceil() as usize;
        boids.iter().zip(0..num_to_despawn).for_each(|(entity,_)|{
            match commands.get_entity(entity){
                Ok(mut entity)=>entity.despawn(),
                _=>{}
            }
        })
    }
}


pub fn boid_screen_wrap(
   mut boids     : Query<&mut Transform, With<Boid>>,
       boid_zone : Query<(&BoidZone, &Transform), Without<Boid>>,
) {
    let Ok((zone_size, zone_position)) = boid_zone.single() else {return};
    let left   = zone_position.translation.x - zone_size.width/2.0;
    let right  = zone_position.translation.x + zone_size.width/2.0;
    let top    = zone_position.translation.y - zone_size.height/2.0;
    let bottom = zone_position.translation.y + zone_size.height/2.0;
    boids.iter_mut().for_each(|mut boid_transform| {
        if boid_transform.translation.x > right {
            boid_transform.translation.x = left;
        } else if boid_transform.translation.x < left {
            boid_transform.translation.x = right;
        }
    
        if boid_transform.translation.y > bottom {
            boid_transform.translation.y = top;
        } else if boid_transform.translation.y < top {
            boid_transform.translation.y = bottom;
        }
    });
}


pub fn boid_move_forward(mut q:Query<(&Boid, &mut Transform), With<Boid>>, config:Res<BoidConfig>){
    q.iter_mut().for_each(|(b, mut t)|{
        let speed = b.speed + config.boid_base_speed;
        t.translation.x += speed*b.facing.cos();
        t.translation.y += speed*b.facing.sin();
        t.rotation = Quat::from_rotation_z(b.facing);
    })
}


#[derive(Clone, Copy)]
pub struct BoidThoughts {
    group_facing:Vec2,
    member_avoidance:Option<Vec2>,
    group_centroid:Vec2,
}

pub fn boid_rotate_to_face_group(
    mut boids:Query<(Entity, &mut Boid, &Transform)>,
    kdtree: Res<KDTree2<Boid>>,
    config: Res<BoidConfig>,
) {
    
    let new_facings:Vec<f32> = boids.iter().map(|(entity, boid, transform)| {

        let thoughts: Vec<BoidThoughts> = 
            kdtree
            .within_distance(transform.translation.truncate(), config.boid_vision_radius)
            .into_iter()
            .filter_map(|item| match item {
                (v,Some(e)) if e!=entity => Some((v,e)),
                _ => None
            } )
            .filter_map(|(_other_position, other_entity)|{
            
            let Ok((_, other_boid, other_transform)) =  boids.get(other_entity) else {return None};

            let facing = Vec2::from_angle(boid.facing);
            let ab = (other_transform.translation - transform.translation).truncate();
            
            let distance = ab.length();
            let ab_norm  = ab / distance; // TODO: risk of baNaNas

            // acos(a.b) = theta

            let other_boid_is_visible =  
                   facing.dot(ab_norm).acos().abs() < config.boid_vision_cone_radius_radians
                && distance                         < config.boid_vision_radius;

            let other_boid_is_crowding = distance < config.boid_crowding_radius;
            if other_boid_is_visible {
                Some(BoidThoughts{
                    group_facing     : Vec2::from_angle(other_boid.facing),
                    member_avoidance : if other_boid_is_crowding { Some(-ab_norm) }else{ None },
                    group_centroid   : other_transform.translation.truncate(),
                })
            }else{
                None
            }
        }).collect();
        let thoughts_length = thoughts.len();


        if thoughts_length==0 {return boid.facing} // head empty


        let group_facing  :Vec2 = thoughts.iter().map(|thought|thought.group_facing  ).sum::<Vec2>() / (thoughts_length as f32);
        let group_centroid:Vec2 = thoughts.iter().map(|thought|thought.group_centroid).sum::<Vec2>() / (thoughts_length as f32);
        
        let (member_avoidance_sum, member_avoidance_count) = thoughts.iter()
            .map(|thought| thought.member_avoidance)
            .fold((Vec2::ZERO, 0), |acc, member_avoidance| {
                match member_avoidance {
                    Some(x)=>(acc.0+x,acc.1+1),
                    None=>acc
                }
            });
        
        
        let ab = group_centroid - transform.translation.truncate();
        let face_group_centroid = ab.to_angle();

        let result = lerp_angle(
            lerp_angle(
                boid.facing,
                group_facing.to_angle(),
                config.average_group_direction,
            ),
            face_group_centroid,
            config.average_group_position,
        );
        if member_avoidance_count==0 {
            result
        }else{
            let average_member_avoidance = member_avoidance_sum / (member_avoidance_count as f32);
            lerp_angle(
                result,
                average_member_avoidance.to_angle(),
                config.avoid_crowding
                //0.04
            )
        }
    }).collect();
    
    boids.iter_mut().zip(new_facings).for_each(|((_, mut boid, _), new_facing)|{
        boid.facing = new_facing
    });
}
