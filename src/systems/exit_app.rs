use bevy::{app::AppExit, ecs::event::EventWriter};

pub fn exit_app(mut exit: EventWriter<AppExit>) {
    exit.write(AppExit::Success);
}