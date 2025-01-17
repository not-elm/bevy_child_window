# bevy_child_window

This library provides a way to create a child window in Bevy.

## Supported platforms

| Platform | usable |
|----------|--------|
| Windows  | ❌      |
| MacOS    | ✅      |
| Linux    | ❌      |
| Web      | ❌      |

## Usage

You can create the window as child by adding `ParentWindow` component to the entity.

```rust
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
            resolution: WindowResolution::new(500.0, 500.0),
            ..Default::default()
        }
    ));
}
```

## ChangeLog

Please see [here](./CHANGELOG.md).

## Compatible Bevy versions

| bevy_child_window | bevy |
|-------------------|------|
| 0.1.0 ~           | 0.15 |

## License

This crate is licensed under the MIT License or the Apache License 2.0.

## Contributing

Welcome to contribute by PR and issues!



