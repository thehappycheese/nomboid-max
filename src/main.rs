mod lerp_angle;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};
use lerp_angle::lerp_angle;

mod systems;

mod components;

use bevy::{
    input::common_conditions::input_just_pressed, prelude::*, window::PrimaryWindow
};
use rand::Rng;

use bevy_spatial::{
    AutomaticUpdate,
    kdtree::KDTree2,
    TransformMode,
    SpatialAccess,
    SpatialStructure
};

#[derive(Component, Default)]
struct TrackedByKDTree;
type NNTree = KDTree2<TrackedByKDTree>; // type alias for later


#[derive(Resource)]
struct Config {
    boid_vision_radius:f32,
    boid_crowding_radius:f32,
    boid_vision_cone_radius_radians:f32,
    boid_base_speed:f32,
    avoid_collide:f32,
    face_group:f32,
    group_centroid:f32,
}

impl Default for Config {
    fn default() -> Self {
        Self { 
            boid_vision_radius              : 30.0,
            boid_crowding_radius            : 10.0,
            boid_vision_cone_radius_radians : 110f32.to_radians(),
            boid_base_speed                 : 0.6,
            avoid_collide                   : 0.03,
            face_group                      : 0.03,
            group_centroid                  : 0.03,
        }
    }
}



fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
            // .set(WindowPlugin{
            //     primary_window: Some(Window {
            //         mode: bevy::window::WindowMode::Fullscreen(MonitorSelection::Primary, VideoModeSelection::Current),
                    
            //         ..default()
            //     }),
            //     ..default()
            // })
        )
        .add_plugins(EguiPlugin::default())
        .add_plugins(AutomaticUpdate::<TrackedByKDTree>::new()
            .with_spatial_ds(SpatialStructure::KDTree2)
            .with_frequency(std::time::Duration::from_secs_f32(0.4))
            .with_transform(TransformMode::GlobalTransform))
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .insert_resource(Config::default())
        .add_systems(Startup, setup)
        
        .add_systems(FixedUpdate,
        (
            esc_to_exit.run_if(input_just_pressed(KeyCode::Escape)),
            boid_rotate_to_face_group,
            boid_move_forward,
            boid_screen_wrap,
        ))
        .add_systems(EguiPrimaryContextPass, ui_example_system)
        
        .run();
}

/// Player component
#[derive(Component)]
struct Player {}

#[derive(Component)]
struct Boid {
    facing:f32,
    speed:f32,
}


fn esc_to_exit(mut exit: EventWriter<AppExit>) {
    exit.write(AppExit::Success);
}

fn ui_example_system(
    mut contexts: EguiContexts,
    mut exit: EventWriter<AppExit>,
    mut config: ResMut<Config>,
) -> Result {
    egui::Window::new("NomBoids-Max").show(contexts.ctx_mut()?, |ui| {
        ui.add(egui::Slider::new(&mut config.boid_vision_radius  , 0.0..=50.0).text("Vision Radius"));
        ui.add(egui::Slider::new(&mut config.boid_crowding_radius, 0.0..=50.0).text("Crowding Radius"));
        ui.add(egui::Slider::new(&mut config.boid_base_speed     , -1.0..=3.0).text("Base Speed"));
        ui.add(egui::Slider::new(&mut config.boid_vision_cone_radius_radians, 0.0..=std::f32::consts::PI).text("Vision Cone Radians"));
        ui.add(egui::Slider::new(&mut config.avoid_collide , 0.0..=0.06).text("avoid_collide"));
        ui.add(egui::Slider::new(&mut config.face_group    , 0.0..=0.06).text("face_group"));
        ui.add(egui::Slider::new(&mut config.group_centroid, 0.0..=0.06).text("group_centroid"));
        ui.spacing();
        if ui.button("Exit").clicked() {
            exit.write(AppExit::Success);
        }
    });
    Ok(())
}


fn setup(
    mut commands: Commands, asset_server: Res<AssetServer>,
    window:Query<&Window, With<PrimaryWindow>>,
) {
    let window = window.single().unwrap();
    let window_width = window.width();
    let window_height = window.height();
    let mut rng = rand::rng();
    let fish = asset_server.load("fish.png");

    commands.spawn(Camera2d);
    


    for _ in 0..4_000 {
        commands.spawn((
            Boid{
                facing:rng.random::<f32>()*3.141*2.0,
                speed:rng.random::<f32>()*0.4,
            },
            Sprite::from_image(fish.clone()),
            Transform::from_xyz(
                (rng.random::<f32>()*1.0-0.5)*window_width,
                (rng.random::<f32>()*1.0-0.5)*window_height,
                0.0
            ).with_scale((0.1,0.1,0.1).into()),
            TrackedByKDTree,
            //ProximityGridStatus{inserted_at:None},
        ));
    }
}



fn boid_screen_wrap(
   mut boids: Query<&mut Transform, With<Boid>>,
   camera: Single<(&Camera, &GlobalTransform)>,
) {
   let (camera, _camera_transform) = *camera;
   let Some(viewport_size) = camera.logical_viewport_size() else { return };
   let half_width = viewport_size.x / 2.0;
   let half_height = viewport_size.y / 2.0;
   
   boids.iter_mut().for_each(|mut transform| {
       if transform.translation.x > half_width {
           transform.translation.x = -half_width;
       } else if transform.translation.x < -half_width {
           transform.translation.x = half_width;
       }
       
       if transform.translation.y > half_height {
           transform.translation.y = -half_height;
       } else if transform.translation.y < -half_height {
           transform.translation.y = half_height;
       }
   });
}


fn boid_move_forward(mut q:Query<(&Boid, &mut Transform), With<Boid>>, config:Res<Config>){
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

fn boid_rotate_to_face_group(
    mut boids:Query<(Entity, &mut Boid, &Transform)>,
    kdtree: Res<NNTree>,
    config: Res<Config>,
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
                config.face_group,
                //0.02
            ),
            face_group_centroid,
            //0.03
            config.group_centroid,
        );
        if member_avoidance_count==0 {
            result
        }else{
            let average_member_avoidance = member_avoidance_sum / (member_avoidance_count as f32);
            lerp_angle(
                result,
                average_member_avoidance.to_angle(),
                config.avoid_collide
                //0.04
            )
        }
    }).collect();
    
    boids.iter_mut().zip(new_facings).for_each(|((_, mut boid, _), new_facing)|{
        boid.facing = new_facing
    });
}
