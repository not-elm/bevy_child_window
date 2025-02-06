//! This example demonstrates how to create a child window rendered with a webview.

use bevy::prelude::*;
use bevy::window::{PrimaryWindow, WindowResolution};
use bevy_child_window::prelude::*;
use bevy_webview_wry::prelude::{WebviewUri, WebviewWryPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            ChildWindowPlugin,
            WebviewWryPlugin::default()
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
        WebviewUri::new("https://bevyengine.org/"),
        Window {
            title: "Child Window".to_string(),
            resizable: true,
            resolution: WindowResolution::new(500.0, 500.0),
            ..Default::default()
        }
    ));
}