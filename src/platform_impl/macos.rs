use bevy::hierarchy::Parent;
use bevy::math::{IVec2, Rect};
use bevy::prelude::{Entity, EventReader, NonSend, Query, Vec2, WindowMoved, With, Without};
use bevy::window::Window;
use bevy::winit::WinitWindows;
use objc2::rc::{Id, Retained};
use objc2_app_kit::{NSView, NSWindow, NSWindowOrderingMode};
use winit::dpi::PhysicalPosition;
#[allow(deprecated)]
use winit::raw_window_handle::HasRawWindowHandle;
use winit::raw_window_handle::RawWindowHandle;

pub fn set_parent(
    child: &winit::window::Window,
    parent: &winit::window::Window,
){
    let Some(child_window) = obtain_ns_window(child) else{
        return;
    };
    let Some(parent_window) = obtain_ns_window(parent) else{
       return;
    };
    unsafe {
        parent_window.addChildWindow_ordered(&child_window, NSWindowOrderingMode::NSWindowAbove);
    }
}

pub fn fit(
    mut er: EventReader<WindowMoved>,
    winit_windows: NonSend<WinitWindows>,
     parents: Query<Entity, (With<Window>, Without<Parent>)>,
    children: Query<(Entity, &Parent), With<Window>>,
){
    for e in er.read(){
        let Ok((child_entity, parent)) = children.get(e.window) else{
            continue;
        };
        let Ok(parent_entity) = parents.get(parent.get()) else{
            continue;
        };
        let Some(child_window) = winit_windows.get_window(e.window) else{
            continue;
        };
        let Some(parent_window) = winit_windows.get_window(parent_entity) else{
            continue;
        };
        fit_window_position(parent_window, child_window);
    }
}

fn obtain_ns_window(
    window: &winit::window::Window,
) -> Option<Retained<NSWindow>>{
    let ns_window = window.raw_window_handle().ok()?;
    if let RawWindowHandle::AppKit(handle) = ns_window {
        let ns_ptr = handle.ns_view.as_ptr();
        let ns_view: Id<NSView> = unsafe { Id::retain(ns_ptr.cast())? };
        ns_view.window()
    } else{
            None
        }
}

fn fit_window_position(
    parent_window: &winit::window::Window,
    child_window: &winit::window::Window,
){
    let Ok(child_pos) = child_window.outer_position() else{
        return;;
    };
    let Ok(parent_pos) = parent_window.outer_position() else{
        return;;
    };
    let child_size = child_window.outer_size().cast::<i32>();
    let parent_size = parent_window.outer_size().cast::<i32>();
    let p = IVec2::new(parent_pos.x + parent_size.width, parent_pos.y + parent_size.height);
    let c = IVec2::new(child_pos.x + child_size.width, child_pos.y + child_size.height);
    if child_pos.x < parent_pos.x{
        child_window.set_outer_position(PhysicalPosition::new(parent_pos.x, child_pos.y));
    }
}