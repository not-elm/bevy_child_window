use crate::platform_impl::macos::ResizeDirection;
use crate::ParentWindow;
use bevy::math::{Rect, Vec2};
use bevy::prelude::{Entity, NonSend, Query, With};
use bevy::window::Window;
use bevy::winit::WinitWindows;
use objc2_app_kit::{NSEvent, NSWindow};
use objc2_foundation::{NSInteger, NSPoint};
use std::sync::mpsc::{Receiver, Sender};
use winit::window::{Cursor, CursorIcon};

#[derive(Debug)]
pub struct RequestCursorChange {
    pub parent_window: Entity,
    pub cursor_icon: CursorIcon,
}

pub struct RequestCursorChangeSender(pub Sender<RequestCursorChange>);

pub struct RequestCursorChangeReceiver(pub Receiver<RequestCursorChange>);

pub fn change_cursor_system(
    rx: NonSend<RequestCursorChangeReceiver>,
    winit_windows: NonSend<WinitWindows>,
    child_windows: Query<(Entity, &ParentWindow), With<Window>>,
) {
    if let Ok(request) = rx.0.try_recv() {
        if let Some(parent) = winit_windows.get_window(request.parent_window) {
            parent.set_cursor(Cursor::Icon(request.cursor_icon));
        }
        for child in child_windows
            .iter()
            .filter(|(_, parent)| parent.0 == request.parent_window) {
            let Some(winit_window) = winit_windows.get_window(child.0) else {
                continue;
            };
            winit_window.set_cursor(Cursor::Icon(request.cursor_icon));
        }
    }
}

pub unsafe fn current_cursor_icon(
    parent: &NSWindow,
    e: &NSEvent,
) -> (NSInteger, CursorIcon) {
    match find_hit_resizing_window(parent, e) {
        Some((child, ResizeDirection::Left)) => {
            (child, CursorIcon::WResize)
        }
        Some((child, ResizeDirection::Right)) => {
            (child, CursorIcon::EResize)
        }
        Some((child, ResizeDirection::Top)) => {
            (child, CursorIcon::NResize)
        }
        Some((child, ResizeDirection::Bottom)) => {
            (child, CursorIcon::SResize)
        }
        Some((child, ResizeDirection::TopLeft)) => {
            (child, CursorIcon::NwResize)
        }
        Some((child, ResizeDirection::TopRight)) => {
            (child, CursorIcon::NeResize)
        }
        Some((child, ResizeDirection::BottomLeft)) => {
            (child, CursorIcon::SwResize)
        }
        Some((child, ResizeDirection::BottomRight)) => {
            (child, CursorIcon::SeResize)
        }
        None => {
            (parent.windowNumber(), CursorIcon::Default)
        }
    }
}

pub fn to_resize_direction(icon: &CursorIcon) -> Option<ResizeDirection> {
    match icon {
        CursorIcon::WResize => Some(ResizeDirection::Left),
        CursorIcon::EResize => Some(ResizeDirection::Right),
        CursorIcon::NResize => Some(ResizeDirection::Top),
        CursorIcon::SResize => Some(ResizeDirection::Bottom),
        _ => None,
    }
}

pub unsafe fn find_hit_resizing_window(
    parent: &NSWindow,
    e: &NSEvent,
) -> Option<(NSInteger, ResizeDirection)> {
    let within_parent = parent.windowNumber() == e.windowNumber();

    let children = parent.childWindows()?;
    let mut children = children.iter().collect::<Vec<_>>();
    children.sort_by_key(|c| c.orderedIndex());

    for child in children.iter().rev() {
        let frame = child.frame();
        let mouse_pos = if within_parent {
            let pos = e.locationInWindow();
            NSPoint::new(pos.x - frame.origin.x, pos.y - frame.origin.y)
        } else {
            e.locationInWindow()
        };

        let center = Vec2::new(
            ((frame.max().x - frame.min().x) / 2.) as f32,
            ((frame.max().y - frame.min().y) / 2.) as f32,
        );
        let size = Vec2::new(
            (frame.size.width + OUTER_L) as f32,
            (frame.size.height + OUTER_L) as f32,
        );

        if !Rect::from_center_size(center, size).contains(Vec2::new(mouse_pos.x as f32, mouse_pos.y as f32)) {
            continue;
        }
        if let Some(dir) = check_resizable_direction(
            frame.size.width,
            frame.size.height,
            mouse_pos.x,
            mouse_pos.y,
        ) {
            return Some((child.windowNumber(), dir));
        }
    }

    None
}

const INNER_L: f64 = 10.0;
const OUTER_L: f64 = 20.0;

fn check_resizable_direction(
    w: f64,
    h: f64,
    x: f64,
    y: f64,
) -> Option<ResizeDirection> {
    fn is_in_horizontal_resizable_space(x: f64) -> bool {
        if x < 0. {
            x.abs() <= OUTER_L
        } else {
            x <= INNER_L
        }
    }

    fn is_in_vertical_resizable_space(y: f64) -> bool {
        if y < 0. {
            y.abs() <= INNER_L
        } else {
            y <= OUTER_L
        }
    }

    let is_right = is_in_horizontal_resizable_space(w - x);
    let is_left = is_in_horizontal_resizable_space(x);
    let is_top = is_in_vertical_resizable_space(y - h);
    let is_bottom = is_in_vertical_resizable_space(y);
    if y.abs() <= OUTER_L && x <= OUTER_L {
        Some(ResizeDirection::TopLeft)
    } else if is_top && is_right {
        Some(ResizeDirection::TopRight)
    } else if is_bottom && is_left {
        Some(ResizeDirection::BottomLeft)
    } else if is_bottom && is_right {
        Some(ResizeDirection::BottomRight)
    } else if is_bottom {
        Some(ResizeDirection::Bottom)
    } else if is_left {
        Some(ResizeDirection::Left)
    } else if is_right {
        Some(ResizeDirection::Right)
    } else if is_top {
        Some(ResizeDirection::Top)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::platform_impl::macos::cursor::{check_resizable_direction, INNER_L, OUTER_L};
    use crate::platform_impl::macos::ResizeDirection;

    #[test]
    fn bottom() {
        assert_eq!(check_resizable_direction(
            100.0,
            100.0,
            30.,
            0.0,
        ), Some(ResizeDirection::Bottom));
    }

    #[test]
    fn left() {
        assert_eq!(check_resizable_direction(
            100.0,
            100.0,
            0.,
            30.0,
        ), Some(ResizeDirection::Left));
    }

    #[test]
    fn left_outer() {
        assert_eq!(check_resizable_direction(
            100.0,
            100.0,
            -OUTER_L,
            30.0,
        ), Some(ResizeDirection::Left));
    }

    #[test]
    fn un_change_left_outer() {
        assert_eq!(check_resizable_direction(
            100.0,
            100.0,
            -OUTER_L - 1.0,
            30.0,
        ), None);
    }

    #[test]
    fn right() {
        assert_eq!(check_resizable_direction(
            100.0,
            100.0,
            100.,
            30.0,
        ), Some(ResizeDirection::Right));
    }

    #[test]
    fn top() {
        assert_eq!(check_resizable_direction(
            100.0,
            100.0,
            30.,
            100.0,
        ), Some(ResizeDirection::Top));
    }

    #[test]
    fn top_outer() {
        assert_eq!(check_resizable_direction(
            100.0,
            100.0,
            INNER_L + 1.0,
            100. + OUTER_L,
        ), Some(ResizeDirection::Top));
    }

    #[test]
    fn top_left() {
        assert_eq!(check_resizable_direction(
            100.0,
            100.0,
            0.,
            100.,
        ), Some(ResizeDirection::TopLeft));
    }
}