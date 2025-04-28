use objc2::rc::Retained;
use objc2::runtime::{NSObject, NSObjectProtocol};
use objc2::{define_class, msg_send, DefinedClass, MainThreadOnly};
use objc2_app_kit::{NSWindow, NSWindowDelegate};
use objc2_foundation::{MainThreadMarker, NSNotification, NSPoint, NSRect, NSSize};
use std::cell::Cell;


pub struct ChildWindowIVars {
    window_origin: Cell<NSRect>,
    dir: Cell<Option<ResizeDirection>>,
}

#[derive(Copy, Clone)]
pub struct ResizeDirection {
    left: bool,
    right: bool,
    top: bool,
    bottom: bool,
}

define_class! {
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = ChildWindowIVars]
    #[name = "ChildWindowDelegate"]
    pub struct ChildWindowDelegate;

    unsafe impl NSObjectProtocol for ChildWindowDelegate {}
    
    unsafe impl NSWindowDelegate for ChildWindowDelegate {
        #[inline]
        #[unsafe(method(windowWillStartLiveResize:))]
        unsafe fn window_will_start_live_resize(&self, notification: &NSNotification){
            on_resize_start(self.ivars(), notification);
        }

        #[inline]
        #[unsafe(method(windowDidResize:))]
        unsafe fn window_did_resize(&self, notification: &NSNotification){
            on_did_resize(self.ivars(), notification);
        }

        #[inline]
        #[unsafe(method(windowWillResize:toSize:))]
        unsafe fn window_will_resize(&self, window: &NSWindow, proposed_size: NSSize) -> NSSize {
            on_will_resize(self.ivars(), window, proposed_size)
        }
    }
}

impl ChildWindowDelegate {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc();
        let this = this.set_ivars(ChildWindowIVars {
            window_origin: Cell::new(NSRect::new(NSPoint::new(0., 0.), NSSize::new(0., 0.))),
            dir: Cell::new(None),
        });
        unsafe { msg_send![super(this), init] }
    }
}

#[inline]
unsafe fn on_resize_start(
    i_vars: &ChildWindowIVars,
    notification: &NSNotification,
) {
    let Some(obj) = notification.object() else {
        return;
    };
    let ns_window: Retained<NSWindow> = Retained::cast_unchecked(obj);
    i_vars.window_origin.set(ns_window.frame());
    i_vars.dir.set(None);
}

#[inline]
unsafe fn on_did_resize(
    i_vars: &ChildWindowIVars,
    notification: &NSNotification,
) {
    if i_vars.dir.get().is_some() {
        return;
    }
    let Some(obj) = notification.object() else {
        return;
    };

    let ns_window: Retained<NSWindow> = Retained::cast_unchecked(obj);
    let start = i_vars.window_origin.get();
    let current = ns_window.frame();
    i_vars.dir.set(Some(ResizeDirection {
        left: is_left(start, current),
        right: is_right(start, current),
        top: is_top(start, current),
        bottom: is_bottom(start, current),
    }));
    i_vars.window_origin.set(ns_window.frame());
}

fn is_left(
    start_rect: NSRect,
    current: NSRect,
) -> bool {
    1e-6 < (start_rect.origin.x - current.origin.x).abs()
}

fn is_right(
    start_rect: NSRect,
    current: NSRect,
) -> bool {
    1e-6 < (start_rect.size.width - current.size.width).abs()
}

fn is_top(
    start_rect: NSRect,
    current: NSRect,
) -> bool {
    1e-6 < (start_rect.size.height - current.size.height).abs()
}

fn is_bottom(
    start_rect: NSRect,
    current: NSRect,
) -> bool {
    1e-6 < (start_rect.origin.y - current.origin.y).abs()
}

#[inline]
fn on_will_resize(
    i_vars: &ChildWindowIVars,
    window: &NSWindow,
    mut size: NSSize,
) -> NSSize {
    let Some(dir) = i_vars.dir.get() else {
        return size;
    };

    let parent_window = unsafe { window.parentWindow() };
    let Some(parent_window) = parent_window else {
        return size;
    };

    let parent_frame = parent_window.contentRectForFrameRect(parent_window.frame());
    let child_frame = window.frame();
    if dir.left {
        let d = size.width - child_frame.size.width;
        if 0. < d && (child_frame.origin.x - d) < parent_frame.origin.x {
            size.width = child_frame.size.width + (child_frame.origin.x - parent_frame.origin.x);
        }
    } else if dir.right {
        let d = size.width - child_frame.size.width;
        if 0. < d && parent_frame.max().x < (child_frame.max().x + d) {
            size.width = parent_frame.max().x - child_frame.origin.x;
        }
    }

    if dir.bottom {
        let d = size.height - child_frame.size.height;
        if 0. < d && (child_frame.min().y - d) < parent_frame.min().y {
            size.height = child_frame.size.height + (child_frame.origin.y - parent_frame.origin.y);
        }
    } else if dir.top {
        let d = size.height - child_frame.size.height;
        if 0. < d && parent_frame.max().y < (child_frame.max().y + d) {
            size.height = parent_frame.max().y - child_frame.origin.y;
        }
    }

    size
}
