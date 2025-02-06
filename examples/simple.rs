//! This example demonstrates how to create a child window.

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy_child_window::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ChildWindowPlugin,
        ))
        .add_systems(Startup, spawn_child_window)
        .run();
}

fn spawn_child_window(
    mut commands: Commands,
    parent: Query<Entity, With<PrimaryWindow>>,
) {
    commands.spawn((
        ParentWindow(parent.single()),
        Window {
            title: "Child Window".to_string(),
            resizable: true,
            resolution: WindowResolution::new(500.0, 500.0),
            ..Default::default()
        }
    ));
}