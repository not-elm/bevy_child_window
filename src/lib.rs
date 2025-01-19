//!  

#![allow(clippy::type_complexity)]

mod platform_impl;
use bevy::app::{App, Plugin};
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::{Component, Entity, Reflect, ReflectComponent, ReflectDefault, ReflectDeserialize, ReflectSerialize};
use serde::{Deserialize, Serialize};

#[allow(missing_docs)]
pub mod prelude {
    pub use crate::ChildWindowPlugin;
    pub use crate::ParentWindow;
}

/// Provides the feature to create a child window
///
/// You can create a child window by inserting [`ParentWindow`].
/// The window belonging to the same entity as its component will be displayed within the area of the parent window.
///
/// # Example
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_child_window::ParentWindow;
///
/// fn spawn_child_window(
///     mut commands: Commands,
///    parent_window: Query<Entity, With<ParentWindow>>,
/// ){
///     commands.spawn((
///         ParentWindow(parent_window.single()),
///         Window::default(),
///    ));
/// }
/// ```
pub struct ChildWindowPlugin;

impl Plugin for ChildWindowPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<ParentWindow>()
            .register_type::<UnInitializeWindow>()
            .add_plugins(platform_impl::ChildWindowPlugin);

        app
            .world_mut()
            .register_component_hooks::<ParentWindow>()
            .on_add(|mut world: DeferredWorld, entity: Entity, _| {
                world.commands().entity(entity).insert(UnInitializeWindow);
            });
    }
}

/// Specifies the entity of the parent window.
///
/// The window belonging to the same entity as this component will be displayed within the area of the parent window.
///
/// # Example
/// ```no_run
/// use bevy::prelude::*;
/// use bevy_child_window::ParentWindow;
///
/// fn spawn_child_window(
///     mut commands: Commands,
///    parent_window: Query<Entity, With<ParentWindow>>,
/// ){
///     commands.spawn((
///         ParentWindow(parent_window.single()),
///         Window::default(),
///    ));
/// }
/// ```
#[derive(Component, Reflect, Serialize, Deserialize)]
#[reflect(Component, Serialize, Deserialize)]
pub struct ParentWindow(pub Entity);

#[derive(Component, Reflect, Serialize, Deserialize, Default)]
#[reflect(Component, Serialize, Deserialize, Default)]
struct UnInitializeWindow;

