//! This example demonstrates how to display different sprites in different windows.

use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::view::RenderLayers;
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
            spawn_sprite_in_primary_window,
            spawn_sprite_in_child_window,
        ))
        .run();
}

fn spawn_child_window(
    mut commands: Commands,
    parent: Query<Entity, With<PrimaryWindow>>,
) {
    let child_window_entity = commands.spawn((
        RenderLayers::layer(1),
        ParentWindow(parent.single()),
        Window {
            title: "Child Window".to_string(),
            resizable: true,
            resolution: WindowResolution::new(500.0, 500.0),
            ..Default::default()
        }
    )).id();

    commands.spawn((
        Camera2d,
        RenderLayers::layer(1),
        Camera {
            target: RenderTarget::Window(WindowRef::Entity(child_window_entity)),
            ..Default::default()
        }
    ));
}

fn spawn_sprite_in_primary_window(
    mut commands: Commands,
) {
    commands.spawn(Camera2d);
    commands.spawn(Sprite {
        custom_size: Some(Vec2::splat(100.)),
        color: Color::WHITE,
        ..Default::default()
    });
}

fn spawn_sprite_in_child_window(
    mut commands: Commands,
) {
    commands.spawn((
        RenderLayers::layer(1),
        Sprite {
            custom_size: Some(Vec2::splat(100.)),
            color: Color::Srgba(Srgba::rgb_u8(0, 0, 255)),
            ..Default::default()
        },
    ));
}


