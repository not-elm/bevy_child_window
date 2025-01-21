mod resize;
mod cursor;

use crate::platform_impl::macos::cursor::{current_cursor_icon, to_resize_direction, RequestCursorChange, RequestCursorChangeSender};
use crate::platform_impl::macos::resize::{resize_on_bottom_edge, resize_on_left_edge, resize_on_right_edge, resize_on_top_edge, resize_on_top_left_edge, resize_on_top_right_edge};
use crate::{ParentWindow, UnInitializeWindow};
use bevy::app::{App, Plugin, Update};
use bevy::prelude::{Commands, Entity, NonSend, Query, ResMut, Resource, With};
use bevy::utils::HashSet;
use bevy::window::Window;
use bevy::winit::WinitWindows;
use block2::RcBlock;
use objc2::ffi::NSInteger;
use objc2::rc::{Id, Retained};
use objc2::ClassType;
use objc2_app_kit::{NSEvent, NSEventMask, NSEventType, NSView, NSWindow, NSWindowOrderingMode, NSWindowStyleMask, NSWindowTitleVisibility};
use objc2_foundation::{CGPoint, CGRect};
use std::cell::Cell;
use std::ptr::{null_mut, NonNull};
use std::sync::mpsc::Sender;
#[allow(deprecated)]
use winit::raw_window_handle::HasRawWindowHandle;
use winit::raw_window_handle::RawWindowHandle;

pub struct ChildWindowPlugin;

impl Plugin for ChildWindowPlugin {
    fn build(&self, app: &mut App) {
        let (sender, receiver) = std::sync::mpsc::channel::<RequestCursorChange>();
        app
            .insert_non_send_resource(RequestCursorChangeSender(sender))
            .insert_non_send_resource(cursor::RequestCursorChangeReceiver(receiver))
            .init_resource::<AlreadyRegisteredWindows>()
            .add_systems(Update, (
                convert_to_child_window,
                cursor::change_cursor_system,
            ));
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum CurrentStatus {
    None,
    Moving(NSInteger),
    Resizing {
        target_window: NSInteger,
        direction: ResizeDirection,
    },
    ResizeCursorVisible {
        target_window: NSInteger,
        direction: ResizeDirection,
    },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ResizeDirection {
    Left,
    Right,
    Top,
    Bottom,
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Resource, Default)]
struct AlreadyRegisteredWindows(HashSet<Entity>);

fn convert_to_child_window(
    mut commands: Commands,
    mut already_registered_windows: ResMut<AlreadyRegisteredWindows>,
    winit_windows: NonSend<WinitWindows>,
    request_cursor_change_sender: NonSend<RequestCursorChangeSender>,
    windows: Query<(Entity, &Window, &ParentWindow), With<UnInitializeWindow>>,
) {
    for (entity, window, ParentWindow(parent_entity)) in windows.iter() {
        let Some(child) = winit_windows.get_window(entity) else {
            continue;
        };
        let Some(parent) = winit_windows.get_window(*parent_entity) else {
            continue;
        };
        let Some(child_window) = obtain_ns_window(child) else {
            return;
        };
        let Some(parent_window) = obtain_ns_window(parent) else {
            return;
        };
        commands.entity(entity).remove::<UnInitializeWindow>();
        settings_windows(window, &child_window, &parent_window);
        if !already_registered_windows.0.contains(parent_entity) {
            unsafe {
                register_ns_event(window.resizable, request_cursor_change_sender.0.clone(), parent_window, *parent_entity);
            }
            already_registered_windows.0.insert(*parent_entity);
        }
    }
}

fn settings_windows(
    window: &Window,
    child_window: &NSWindow,
    parent_window: &NSWindow,
) {
    unsafe {
        parent_window.addChildWindow_ordered(child_window, NSWindowOrderingMode::NSWindowAbove);
    }

    child_window.setMovable(false);
    child_window.setStyleMask(style_mask(window));
    child_window.setTitleVisibility(if window.titlebar_show_title {
        NSWindowTitleVisibility::NSWindowTitleVisible
    } else {
        NSWindowTitleVisibility::NSWindowTitleHidden
    });
}

fn style_mask(window: &Window) -> NSWindowStyleMask {
    let mut mask = NSWindowStyleMask::empty();
    if !window.titlebar_shown {
        return mask;
    }
    if window.titlebar_show_buttons {
        mask |= NSWindowStyleMask::Titled | NSWindowStyleMask::Closable;
    }
    mask
}

unsafe fn register_ns_event(
    resizable: bool,
    request_cursor_change_sender: Sender<RequestCursorChange>,
    parent_window: Retained<NSWindow>,
    parent_window_entity: Entity,
) {
    let status = Cell::new(CurrentStatus::None);
    NSEvent::addLocalMonitorForEventsMatchingMask_handler(
        NSEventMask::LeftMouseDragged | NSEventMask::LeftMouseDown | NSEventMask::LeftMouseUp | NSEventMask::MouseMoved,
        Box::leak(Box::new(RcBlock::new(move |event: NonNull<NSEvent>| {
            let e = &*event.as_ptr();
            match (e.r#type(), status.get()) {
                (NSEventType::LeftMouseDown, CurrentStatus::None) => {
                    transition_to_move(&parent_window, &status, e);
                }
                (NSEventType::LeftMouseDown, CurrentStatus::ResizeCursorVisible {
                    target_window,
                    direction,
                }) => {
                    transition_to_resize(&parent_window, &status, target_window, direction);
                }
                (NSEventType::LeftMouseUp, _) => {
                    status.set(CurrentStatus::None);
                }
                (NSEventType::MouseMoved, CurrentStatus::None | CurrentStatus::ResizeCursorVisible { direction: _, target_window: _ }) if resizable => {
                    let (child_window, cursor_icon) = current_cursor_icon(&parent_window, e);
                    let _ = request_cursor_change_sender.send(RequestCursorChange {
                        parent_window: parent_window_entity,
                        cursor_icon,
                    });
                    if let Some(resize_direction) = to_resize_direction(&cursor_icon) {
                        status.set(CurrentStatus::ResizeCursorVisible {
                            target_window: child_window,
                            direction: resize_direction,
                        });
                    } else {
                        status.set(CurrentStatus::None);
                    }
                }
                (NSEventType::LeftMouseDragged, CurrentStatus::Resizing {
                    target_window,
                    direction: ResizeDirection::Left,
                }) => {
                    resize_on_left_edge(&parent_window, target_window, e.deltaX());
                }
                (NSEventType::LeftMouseDragged, CurrentStatus::Resizing {
                    target_window,
                    direction: ResizeDirection::Right,
                }) => {
                    resize_on_right_edge(&parent_window, target_window, e.deltaX());
                }
                (NSEventType::LeftMouseDragged, CurrentStatus::Resizing {
                    target_window,
                    direction: ResizeDirection::Top,
                }) => {
                    resize_on_top_edge(&parent_window, target_window, e.deltaY());
                }
                (NSEventType::LeftMouseDragged, CurrentStatus::Resizing {
                    target_window,
                    direction: ResizeDirection::Bottom,
                }) => {
                    resize_on_bottom_edge(&parent_window, target_window, e.deltaY());
                }
                (NSEventType::LeftMouseDragged, CurrentStatus::Resizing {
                    target_window,
                    direction: ResizeDirection::TopLeft,
                }) => {
                    resize_on_top_left_edge(
                        &parent_window,
                        target_window,
                        e.deltaX(),
                        e.deltaY(),
                    );
                }
                (NSEventType::LeftMouseDragged, CurrentStatus::Resizing {
                    target_window,
                    direction: ResizeDirection::TopRight,
                }) => {
                    resize_on_top_right_edge(
                        &parent_window,
                        target_window,
                        e.deltaX(),
                        e.deltaY(),
                    );
                }
                (NSEventType::LeftMouseDragged, CurrentStatus::Resizing {
                    target_window,
                    direction: ResizeDirection::BottomLeft,
                }) => {
                    resize_on_left_edge(&parent_window, target_window, e.deltaX());
                    resize_on_bottom_edge(&parent_window, target_window, e.deltaY());
                }
                (NSEventType::LeftMouseDragged, CurrentStatus::Resizing {
                    target_window,
                    direction: ResizeDirection::BottomRight,
                }) => {
                    resize_on_right_edge(&parent_window, target_window, e.deltaX());
                    resize_on_bottom_edge(&parent_window, target_window, e.deltaY());
                }
                (NSEventType::LeftMouseDragged, CurrentStatus::Moving(target_num)) => {
                    let Some(child_window) = find_child_window(&parent_window, target_num) else {
                        return null_mut();
                    };
                    move_child_window(&parent_window, &child_window, e.deltaX(), e.deltaY());
                }
                _ => {}
            }
            event.as_ptr()
        }))),
    );
}

#[inline]
unsafe fn move_child_window(
    parent_window: &NSWindow,
    child_window: &NSWindow,
    delta_x: f64,
    delta_y: f64,
) {
    let c = child_window.frame();
    let p = parent_window.contentRectForFrameRect(parent_window.frame());
    let x = p.origin.x.max(c.origin.x + delta_x);
    let x = x.min(p.origin.x + p.size.width - c.size.width);
    let y = c.origin.y - delta_y;
    let y = p.origin.y.max(y);
    let y = y.min(p.origin.y + p.size.height - c.size.height);

    child_window.setFrame_display(CGRect::new(
        CGPoint::new(x, y),
        c.size,
    ), false);
}

unsafe fn find_child_window(window: &NSWindow, window_num: NSInteger) -> Option<Retained<NSWindow>> {
    for child in window.childWindows()?.iter() {
        if child.windowNumber() == window_num {
            return Some(child.retain());
        }
    }
    None
}

fn obtain_ns_window(
    window: &winit::window::Window,
) -> Option<Retained<NSWindow>> {
    #[allow(deprecated)]
    let ns_window = window.raw_window_handle().ok()?;
    if let RawWindowHandle::AppKit(handle) = ns_window {
        let ns_ptr = handle.ns_view.as_ptr();
        let ns_view: Id<NSView> = unsafe { Id::retain(ns_ptr.cast())? };
        ns_view.window()
    } else {
        None
    }
}

unsafe fn bring_to_front_child_window(
    parent_window: &NSWindow,
    child_window: &NSWindow,
) {
    // parent_window.setIgnoresMouseEvents(true);
    parent_window.removeChildWindow(child_window);
    if let Some(children) = parent_window.childWindows() {
        for child in children.iter() {
            if child.isKeyWindow() {
                child.resignKeyWindow();
            }
        }
    }
    parent_window.addChildWindow_ordered(child_window, NSWindowOrderingMode::NSWindowAbove);
    child_window.becomeKeyWindow();
}

unsafe fn transition_to_move(
    parent_window: &NSWindow,
    status: &Cell<CurrentStatus>,
    e: &NSEvent,
) {
    if let Some(child_window) = find_child_window(parent_window, e.windowNumber()) {
        if child_window.contentRectForFrameRect(child_window.frame()).size.height <= e.locationInWindow().y {
            bring_to_front_child_window(parent_window, &child_window);
            status.set(CurrentStatus::Moving(e.windowNumber()));
        }
    };
}

unsafe fn transition_to_resize(
    parent_window: &NSWindow,
    status: &Cell<CurrentStatus>,
    child_window_num: NSInteger,
    resize_dir: ResizeDirection,
) {
    if let Some(child_window) = find_child_window(parent_window, child_window_num) {
        bring_to_front_child_window(parent_window, &child_window);
        status.set(CurrentStatus::Resizing {
            target_window: child_window_num,
            direction: resize_dir,
        });
    }
}