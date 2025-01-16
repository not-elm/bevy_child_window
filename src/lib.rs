mod platform_impl;

use bevy::app::{App, Plugin, PreUpdate};
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::{Added, Commands, Component, Entity, NonSend, Parent, Query, With};
use bevy::window::Window;
use bevy::winit::WinitWindows;

pub struct BevyChildWindowPlugin;

impl Plugin for BevyChildWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, (
            insert,
            convert_to_child_window,
            platform_impl::fit,
            ));
    }
}

fn insert(
    mut commands: Commands,
    windows: Query<Entity, (Added<Window>, With<Parent>)>,
){
    for entity in windows.iter(){
        commands.entity(entity).insert(UnInitialize);
    }
}

# [derive(Component)]
struct UnInitialize;

fn convert_to_child_window(
    mut commands: Commands,
    winit_windows: NonSend<WinitWindows>,
    windows: Query<(Entity, &Parent), (With<Window>, With<UnInitialize>)>,
){
    for (entity, parent) in windows.iter(){
        let Some(child) = winit_windows.get_window(entity) else{
            continue;
        };
        let Some(parent) = winit_windows.get_window(parent.get()) else{
            continue;
        };
        platform_impl::set_parent(child, parent);
        commands.entity(entity).remove::<UnInitialize>();
    }
}