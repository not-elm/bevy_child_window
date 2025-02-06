use crate::{ParentWindow, UnInitializeWindow};
use bevy::app::{App, Plugin, Update};
use bevy::log::error;
use bevy::math::IVec2;
use bevy::prelude::{any_with_component, Commands, Entity, IntoSystemConfigs, NonSend, Query, Window, With};
use bevy::winit::WinitWindows;
use std::collections::BTreeMap;
use std::ffi::c_void;
use std::sync::Mutex;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{CallWindowProcW, GetWindowRect, SetWindowLongPtrW, GWLP_WNDPROC, SM_CYCAPTION, WM_MOVING, WNDPROC};
use windows::Win32::UI::WindowsAndMessaging::{
    DefWindowProcW, GetAncestor, GetClientRect,
    GetSystemMetrics, SetParent,
};
#[allow(deprecated)]
use winit::raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

/// On Windows, by default, the window's own area is managed within the parent's window area, but the behavior was such that the window position would slightly protrude.
/// Therefore, `WindowProc` is used to force the drag area to be managed.
pub struct ChildWindowPlugin;

impl Plugin for ChildWindowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, convert_to_child_window.run_if(any_with_component::<UnInitializeWindow>));
    }
}

fn convert_to_child_window(
    mut commands: Commands,
    winit_windows: NonSend<WinitWindows>,
    windows: Query<(Entity, &ParentWindow), (With<Window>, With<UnInitializeWindow>)>,
) {
    for (entity, ParentWindow(parent_entity)) in windows.iter() {
        let Some(child) = winit_windows.get_window(entity) else {
            continue;
        };
        let Some(parent) = winit_windows.get_window(*parent_entity) else {
            continue;
        };

        let Some(child_window_handle) = obtain_window_handle(child) else {
            continue;
        };
        let Some(parent_window_handle) = obtain_window_handle(parent) else {
            continue;
        };
        unsafe {
            match SetParent(child_window_handle, Some(parent_window_handle)) {
                Ok(_) => {
                    #[allow(clippy::fn_to_numeric_cast)]
                    let default_window_proc = SetWindowLongPtrW(child_window_handle, GWLP_WNDPROC, window_move_proc as isize);
                    HOOKS.lock().unwrap().insert(
                        child_window_handle.0 as isize,
                        #[allow(clippy::missing_transmute_annotations)]
                        Some(std::mem::transmute(default_window_proc)),
                    );
                    commands.entity(entity).remove::<UnInitializeWindow>();
                }
                Err(e) => error!("{e}")
            }
        }
    }
}

static HOOKS: Mutex<BTreeMap<isize, WNDPROC>> = Mutex::new(BTreeMap::new());

unsafe extern "system" fn window_move_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_MOVING => {
            let Some(parent_window_rect) = obtain_parent_window_rect(hwnd) else {
                return call_default_hook(hwnd, msg, wparam, lparam);
            };
            let Some(window_size) = calc_window_size(hwnd) else {
                return call_default_hook(hwnd, msg, wparam, lparam);
            };
            let Some(frame_size) = calc_frame_size(hwnd, window_size) else {
                return call_default_hook(hwnd, msg, wparam, lparam);
            };

            let mut drag_rect = *(lparam.0 as *mut RECT);
            adjust_drag_rect_width(&mut drag_rect, &parent_window_rect, window_size.x, frame_size.x);
            adjust_drag_rect_height(&mut drag_rect, &parent_window_rect, window_size.y, frame_size.y);
            *(lparam.0 as *mut RECT) = drag_rect;

            call_default_hook(hwnd, msg, wparam, lparam)
        }
        _ => call_default_hook(hwnd, msg, wparam, lparam),
    }
}

unsafe fn obtain_parent_window_rect(hwnd: HWND) -> Option<RECT> {
    let parent = GetAncestor(
        hwnd,
        windows::Win32::UI::WindowsAndMessaging::GET_ANCESTOR_FLAGS(1),
    );

    let mut parent_window_rect = RECT::default();
    GetWindowRect(parent, &mut parent_window_rect).ok()?;
    Some(parent_window_rect)
}

unsafe fn calc_window_size(hwnd: HWND) -> Option<IVec2> {
    let mut window_rect = RECT::default();
    GetWindowRect(hwnd, &mut window_rect).ok()?;
    Some(IVec2::new(
        window_rect.right - window_rect.left,
        window_rect.bottom - window_rect.top,
    ))
}

unsafe fn calc_frame_size(
    hwnd: HWND,
    window_size: IVec2,
) -> Option<IVec2> {
    let mut window_client = RECT::default();
    GetClientRect(hwnd, &mut window_client).ok()?;

    let client_h = window_client.bottom - window_client.top;
    let client_w = window_client.right - window_client.left;
    Some(IVec2::new(
        (window_size.x - client_w) / 2,
        (window_size.y - client_h) / 2,
    ))
}

unsafe fn adjust_drag_rect_width(
    drag_rect: &mut RECT,
    parent_window_rect: &RECT,
    window_width: i32,
    frame_width: i32,
) {
    let max_x = parent_window_rect.right - frame_width;
    let min_x = parent_window_rect.left + frame_width;

    if max_x < drag_rect.right {
        drag_rect.right = max_x;
        drag_rect.left = drag_rect.left.min(drag_rect.right - window_width);
    } else if drag_rect.left < min_x {
        drag_rect.left = min_x;
        drag_rect.right = drag_rect.right.max(drag_rect.left + window_width + frame_width);
    }
}

unsafe fn adjust_drag_rect_height(
    drag_rect: &mut RECT,
    parent_frame: &RECT,
    window_height: i32,
    frame_height: i32,
) {
    let title_bar_height = GetSystemMetrics(SM_CYCAPTION);
    let min_y = parent_frame.top + title_bar_height + frame_height;
    let max_y = parent_frame.bottom - frame_height;

    if drag_rect.top < min_y {
        drag_rect.top = min_y;
        drag_rect.bottom = drag_rect.bottom.max(min_y + window_height);
    } else if max_y < drag_rect.bottom {
        drag_rect.bottom = max_y;
        drag_rect.top = drag_rect.top.min(max_y - window_height);
    }
}

unsafe fn call_default_hook(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if let Some(hook) = HOOKS
        .try_lock()
        .ok()
        .and_then(|hooks| hooks.get(&(hwnd.0 as isize)).copied())
    {
        CallWindowProcW(hook, hwnd, msg, wparam, lparam)
    } else {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

fn obtain_window_handle(window: &winit::window::Window) -> Option<HWND> {
    #[allow(deprecated)]
    let handle = window.raw_window_handle().ok()?;
    match handle {
        RawWindowHandle::Win32(handle) => Some(HWND(handle.hwnd.get() as *mut c_void)),
        _ => None,
    }
}
