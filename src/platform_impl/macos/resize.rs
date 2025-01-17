use crate::platform_impl::macos::find_child_window;
use objc2::ffi::NSInteger;
use objc2_app_kit::NSWindow;
use objc2_foundation::{CGPoint, CGRect, CGSize};

pub unsafe fn resize_on_top_left_edge(
    parent: &NSWindow,
    target_window: NSInteger,
    delta_x: f64,
    delta_y: f64,
) {
    let Some(target_window) = find_child_window(parent, target_window) else {
        return;
    };
    let frame = resize_left_frame(
        parent.frame(),
        target_window.frame(),
        delta_x,
        parent.minSize().width,
    );
    let frame = resize_top_frame(
        parent.contentRectForFrameRect(parent.frame()),
        frame,
        delta_y,
        target_window.minSize().height,
    );
    target_window.setFrame_display(frame, true);
}

pub unsafe fn resize_on_top_right_edge(
    parent: &NSWindow,
    target_window: NSInteger,
    delta_x: f64,
    delta_y: f64,
) {
    let Some(target_window) = find_child_window(parent, target_window) else {
        return;
    };
    let frame = resize_right_frame(
        parent.frame(),
        target_window.frame(),
        delta_x,
        parent.minSize().width,
    );
    let frame = resize_top_frame(
        parent.contentRectForFrameRect(parent.frame()),
        frame,
        delta_y,
        target_window.minSize().height,
    );
    target_window.setFrame_display(frame, true);
}

pub unsafe fn resize_on_left_edge(
    parent: &NSWindow,
    target_window: NSInteger,
    delta_x: f64,
) {
    let Some(target_window) = find_child_window(parent, target_window) else {
        return;
    };
    let frame = resize_left_frame(
        parent.frame(),
        target_window.frame(),
        delta_x,
        parent.minSize().width,
    );
    target_window.setFrame_display(frame, true);
}

pub unsafe fn resize_on_right_edge(
    parent: &NSWindow,
    target_window: NSInteger,
    delta_x: f64,
) {
    let Some(target_window) = find_child_window(parent, target_window) else {
        return;
    };
    let frame = resize_right_frame(
        parent.contentRectForFrameRect(parent.frame()),
        target_window.frame(),
        delta_x,
        target_window.minSize().width,
    );
    target_window.setFrame_display(frame, true);
}

pub unsafe fn resize_on_top_edge(
    parent: &NSWindow,
    target_window: NSInteger,
    delta_y: f64,
) {
    let Some(target_window) = find_child_window(parent, target_window) else {
        return;
    };
    let frame = resize_top_frame(
        parent.contentRectForFrameRect(parent.frame()),
        target_window.frame(),
        delta_y,
        target_window.minSize().height,
    );
    target_window.setFrame_display(frame, true);
}

pub unsafe fn resize_on_bottom_edge(
    parent: &NSWindow,
    target_window: NSInteger,
    delta_y: f64,
) {
    let Some(target_window) = find_child_window(parent, target_window) else {
        return;
    };
    let frame = resize_bottom_frame(
        parent.contentRectForFrameRect(parent.frame()),
        target_window.frame(),
        delta_y,
        target_window.minSize().height,
    );
    target_window.setFrame_display(frame, true);
}

fn resize_left_frame(
    parent: CGRect,
    child: CGRect,
    delta_x: f64,
    min_width: f64,
) -> CGRect {
    if child.size.width - delta_x <= min_width {
        return child;
    }
    let x = child.origin.x + delta_x;
    let x = x.min(parent.origin.x + parent.size.width - min_width);
    let x = x.max(parent.origin.x);
    CGRect::new(
        CGPoint::new(x, child.origin.y),
        CGSize::new(child.size.width + child.origin.x - x, child.size.height),
    )
}

fn resize_right_frame(
    parent: CGRect,
    child: CGRect,
    delta_x: f64,
    min_width: f64,
) -> CGRect {
    let width = (child.size.width + delta_x).max(min_width);
    let width = width.max(min_width);
    let width = width.min(parent.origin.x + parent.size.width - child.origin.x);
    CGRect::new(
        child.origin,
        CGSize::new(width, child.size.height),
    )
}

fn resize_top_frame(
    parent: CGRect,
    child: CGRect,
    delta_y: f64,
    min_height: f64,
) -> CGRect {
    let height = (child.size.height - delta_y).max(min_height);
    let height = height.min(parent.origin.y + parent.size.height - child.origin.y);
    CGRect::new(
        child.origin,
        CGSize::new(child.size.width, height),
    )
}

fn resize_bottom_frame(
    parent: CGRect,
    child: CGRect,
    delta_y: f64,
    min_height: f64,
) -> CGRect {
    if child.size.height + delta_y <= min_height {
        return child;
    }
    let y = parent.origin.y.max(child.origin.y - delta_y);
    CGRect::new(
        CGPoint::new(child.origin.x, y),
        CGSize::new(child.size.width, child.size.height + child.origin.y - y),
    )
}


#[cfg(test)]
mod tests {
    use crate::platform_impl::macos::resize::{resize_right_frame, resize_top_frame};
    use objc2_foundation::{CGPoint, CGRect, CGSize};

    #[test]
    fn resize_right_expand() {
        assert_eq!(resize_right_frame(
            CGRect::new(
                CGPoint::new(0.0, 0.0),
                CGSize::new(100.0, 100.0),
            ),
            CGRect::new(
                CGPoint::new(0.0, 0.0),
                CGSize::new(50.0, 50.0),
            ),
            10.,
            50.,
        ), CGRect::new(
            CGPoint::new(0.0, 0.0),
            CGSize::new(60.0, 50.0),
        ));
    }

    #[test]
    fn resize_right_shrink() {
        assert_eq!(resize_right_frame(
            CGRect::new(
                CGPoint::new(0.0, 0.0),
                CGSize::new(100.0, 100.0),
            ),
            CGRect::new(
                CGPoint::new(0.0, 0.0),
                CGSize::new(50.0, 50.0),
            ),
            -10.,
            30.,
        ), CGRect::new(
            CGPoint::new(0.0, 0.0),
            CGSize::new(40.0, 50.0),
        ));
    }

    #[test]
    fn resize_right_min_width() {
        assert_eq!(resize_right_frame(
            CGRect::new(
                CGPoint::new(0.0, 0.0),
                CGSize::new(100.0, 100.0),
            ),
            CGRect::new(
                CGPoint::new(0.0, 0.0),
                CGSize::new(50.0, 50.0),
            ),
            -50.,
            30.,
        ), CGRect::new(
            CGPoint::new(0.0, 0.0),
            CGSize::new(30.0, 50.0),
        ));
    }

    #[test]
    fn resize_right_restrict_parent_width() {
        assert_eq!(resize_right_frame(
            CGRect::new(
                CGPoint::new(0.0, 0.0),
                CGSize::new(100.0, 100.0),
            ),
            CGRect::new(
                CGPoint::new(50.0, 0.0),
                CGSize::new(50.0, 50.0),
            ),
            120.,
            30.,
        ), CGRect::new(
            CGPoint::new(50.0, 0.0),
            CGSize::new(50.0, 50.0),
        ));
    }

    #[test]
    fn resize_top_expand() {
        assert_eq!(resize_top_frame(
            CGRect::new(
                CGPoint::new(0.0, 0.0),
                CGSize::new(100.0, 100.0),
            ),
            CGRect::new(
                CGPoint::new(50.0, 0.0),
                CGSize::new(10.0, 10.0),
            ),
            -20.,
            20.,
        ), CGRect::new(
            CGPoint::new(50.0, 0.0),
            CGSize::new(10.0, 30.0),
        ));
    }

    #[test]
    fn resize_top_strict_parent_size() {
        assert_eq!(resize_top_frame(
            CGRect::new(
                CGPoint::new(0.0, 0.0),
                CGSize::new(100.0, 100.0),
            ),
            CGRect::new(
                CGPoint::new(50.0, 50.0),
                CGSize::new(10.0, 10.0),
            ),
            -100.,
            20.,
        ), CGRect::new(
            CGPoint::new(50.0, 50.0),
            CGSize::new(10.0, 50.0),
        ));
    }
}