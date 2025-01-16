use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy_child_window::BevyChildWindowPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            BevyChildWindowPlugin,
            ))
        .add_systems(Startup, spawn_child_window)
        .run();
}

fn spawn_child_window(
    mut commands: Commands,
    parent: Query<Entity, With<PrimaryWindow>>,
){
    commands.entity(parent.single()).with_child(Window{
        title: "Child Window".to_string(),
        resolution: WindowResolution::new(200., 200.),
        ..default()
    });
}