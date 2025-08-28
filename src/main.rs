mod components;
mod systems;
mod plugins;

mod util;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiPrimaryContextPass};

use bevy::{
    input::common_conditions::input_just_pressed, prelude::*, window::WindowMode,
};

use bevy_pancam::{DirectionKeys, PanCam, PanCamPlugin};
use plugins::boids::{BoidConfig, BoidPlugin, BoidZone};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanCamPlugin::default()
        ))
        .add_plugins(BoidPlugin)
        .add_plugins(EguiPlugin::default())
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        .add_systems(Startup, setup)
        .add_systems(FixedUpdate, (
            systems::exit_app.run_if(input_just_pressed(KeyCode::Escape)),
        ))
        .add_systems(EguiPrimaryContextPass, config_ui)
        .run();
}

fn setup(
    mut commands: Commands
) {
    commands.spawn((
        Camera2d,
        PanCam {
            grab_buttons: vec![MouseButton::Left, MouseButton::Middle], // which buttons should drag the camera
            move_keys: DirectionKeys {      // the keyboard buttons used to move the camera
                up:    vec![KeyCode::KeyQ], // initalize the struct like this or use the provided methods for
                down:  vec![KeyCode::KeyW], // common key combinations
                left:  vec![KeyCode::KeyE],
                right: vec![KeyCode::KeyR],
            },
            speed: 400., // the speed for the keyboard movement
            enabled: true, // when false, controls are disabled. See toggle example.
            zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
            min_scale: 0.1, // prevent the camera from zooming too far in
            max_scale: 10., // prevent the camera from zooming too far out
            min_x: -1600.0, // minimum x position of the camera window
            max_x: 1600.0, // maximum x position of the camera window
            min_y: -900.0, // minimum y position of the camera window
            max_y: 900.0, // maximum y position of the camera window
        },
    ));
    commands.spawn((
        BoidZone{
            width:1600.0,
            height:900.0,
        },
        Transform::from_xyz(0.0, 0.0, 0.0)
    ));
}

fn config_ui(
    mut contexts: EguiContexts,
    mut exit: EventWriter<AppExit>,
    mut config: ResMut<BoidConfig>,
    mut windows: Query<&mut Window>,
) -> Result {
    egui::Window::new("NomBoids-Max").show(contexts.ctx_mut()?, |ui| {
        ui.add(egui::Slider::new(&mut config.boid_vision_radius  , 0.0..=50.0).text("Vision Radius"));
        ui.add(egui::Slider::new(&mut config.boid_crowding_radius, 0.0..=50.0).text("Crowding Radius"));
        ui.add(egui::Slider::new(&mut config.boid_base_speed     , -1.0..=3.0).text("Base Speed"));
        ui.add(egui::Slider::new(&mut config.boid_vision_cone_radius_radians, 0.0..=std::f32::consts::PI).text("Vision Cone Radians"));
        ui.add(egui::Slider::new(&mut config.avoid_crowding , 0.0..=0.06).text("Avoid Crowding"));
        ui.add(egui::Slider::new(&mut config.average_group_direction    , 0.0..=0.06).text("Average Group Direction"));
        ui.add(egui::Slider::new(&mut config.average_group_position, 0.0..=0.06).text("Average Group Position"));
        ui.add(egui::Slider::new(&mut config.target_boid_population, 0..=10000).text("Target Population"));
        ui.add_space(10.0);
        if ui.button("Exit").clicked() {
            exit.write(AppExit::Success);
        }
        ui.add_space(10.0);
        if let Ok(mut window) = windows.single_mut(){
            if ui.button(
                match window.mode {
                    WindowMode::Windowed=>"Enter Fullscreen",
                    WindowMode::BorderlessFullscreen(_)| WindowMode::Fullscreen(_,_) =>"Exit Fullscreen"
                }
            ).clicked() {
                window.mode = match window.mode {
                    WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
                    _ => WindowMode::Windowed,
                }
            }
        }
        
    });
    Ok(())
}


