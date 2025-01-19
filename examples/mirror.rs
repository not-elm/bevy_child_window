//! This example demonstrates how to create a  mirror window.

use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::window::{PrimaryWindow, WindowRef, WindowResolution};
use bevy_child_window::prelude::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ChildWindowPlugin,
        ))
        .add_systems(Startup, (
            spawn_child_window,
            spawn_sprite,
        ))
        .add_systems(Update, rotate)
        .run();
}

fn spawn_child_window(
    mut commands: Commands,
    parent: Query<Entity, With<PrimaryWindow>>,
) {
    let entity = commands.spawn((
        ParentWindow(parent.single()),
        Window {
            title: "Child Window".to_string(),
            resolution: WindowResolution::new(500.0, 500.0),
            ..Default::default()
        }
    )).id();
    commands.spawn((
        Camera2d,
        Camera {
            target: RenderTarget::Window(WindowRef::Entity(entity)),
            ..default()
        },
    ));
}

fn spawn_sprite(
    mut commands: Commands,
) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite {
        custom_size: Some(Vec2::splat(100.)),
        color: Color::WHITE,
        ..default()
    });
}

fn rotate(
    mut sprite: Query<&mut Transform, With<Sprite>>,
    time: Res<Time>,
) {
    for mut transform in sprite.iter_mut() {
        transform.rotate_z(time.delta_secs());
    }
}