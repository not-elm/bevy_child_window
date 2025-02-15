use bevy::app::Plugin;

/// For unsupported platforms, this plugin is used.
///
/// This is only to avoid compile errors and doesn't actually do anything.
pub struct ChildWindowPlugin;

impl Plugin for ChildWindowPlugin {
    fn build(&self, _app: &mut bevy::app::App) {}
}