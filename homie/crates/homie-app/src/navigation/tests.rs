use super::render::relative_parent;
use super::*;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

use gpui::{Entity, ScrollDelta, ScrollWheelEvent, TestAppContext, point};

#[test]
fn relative_parent_abbreviates_home_like_swift() {
    let home = std::env::var_os("HOME").map(PathBuf::from).unwrap();
    assert_eq!(relative_parent(&home.join("project")), "~");
    assert_eq!(relative_parent(&home.join("fun/project")), "~/fun");
    assert_eq!(relative_parent(Path::new("/tmp/project")), "/tmp");
}

#[test]
fn debounce_is_the_swift_value() {
    assert_eq!(RANK_DEBOUNCE, std::time::Duration::from_millis(25));
}

#[test]
fn scrolling_to_a_row_counts_the_section_headers_above_it() {
    // Five actions, then sessions: "Actions" header, five rows, "Sessions"
    // header, then the session rows.
    assert_eq!(row_child_index(0, Some(5)), 1);
    assert_eq!(row_child_index(4, Some(5)), 5);
    assert_eq!(row_child_index(5, Some(5)), 7);
    // No actions matched: only the "Sessions" header sits above row 0.
    assert_eq!(row_child_index(0, Some(0)), 1);
    // A searched Quick Open list has no headers at all.
    assert_eq!(row_child_index(3, None), 3);
}

fn layout(width: f32, height: f32) -> OverlayLayout {
    OverlayLayout::for_viewport(gpui::size(px(width), px(height)))
}

#[test]
fn overlay_never_grows_past_the_window_it_floats_in() {
    for (width, height) in [
        (1100.0, 700.0),
        (1800.0, 1100.0),
        (900.0, 495.0),
        (600.0, 360.0),
    ] {
        let layout = layout(width, height);
        let total = layout.top_inset + px(SEARCH_HEIGHT + 1.0) + layout.list_height;
        assert!(
            total <= px(height),
            "{width}x{height} overflows by {:?}",
            total - px(height)
        );
        assert!(layout.width <= px(width));
    }
}

#[test]
fn the_list_uses_the_height_the_window_actually_has() {
    // The old fixed 400pt list wasted a tall window and overflowed a short
    // one; both directions now track the viewport.
    assert!(layout(1400.0, 1100.0).list_height > px(400.0));
    assert!(layout(900.0, 495.0).list_height < px(400.0));
    // Beyond the cap the surface stops growing rather than becoming a wall.
    assert_eq!(layout(1600.0, 3000.0).list_height, px(MAX_LIST_HEIGHT));
    // A window too short for the minimum list gives up its top inset first,
    // down to the floor.
    let cramped = layout(800.0, 180.0);
    assert!(cramped.top_inset < px(180.0 / 12.0));
    assert_eq!(cramped.list_height, px(MIN_LIST_HEIGHT));
    assert_eq!(layout(800.0, 150.0).top_inset, px(MIN_TOP_INSET));
}

struct WheelHarness {
    overlay: Entity<NavigationOverlay>,
    background_scrolls: Arc<AtomicUsize>,
}

impl Render for WheelHarness {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let background_scrolls = Arc::clone(&self.background_scrolls);
        div()
            .size_full()
            .child(div().absolute().inset_0().on_scroll_wheel(move |_, _, _| {
                background_scrolls.fetch_add(1, AtomicOrdering::Relaxed);
            }))
            .child(crate::root::cached_window_overlay(self.overlay.clone()))
    }
}

#[gpui::test]
fn modal_backdrop_consumes_wheel_events(cx: &mut TestAppContext) {
    let runtime = Arc::new(StoreRuntime::inert());
    let background_scrolls = Arc::new(AtomicUsize::new(0));
    let scroll_probe = Arc::clone(&background_scrolls);
    let (_view, cx) = cx.add_window_view(move |_window, cx| {
        let overlay = cx.new(|cx| NavigationOverlay::opened_for_test(runtime, cx));
        WheelHarness {
            overlay,
            background_scrolls: scroll_probe,
        }
    });

    cx.simulate_event(ScrollWheelEvent {
        position: point(px(8.0), px(320.0)),
        delta: ScrollDelta::Pixels(point(px(0.0), px(-40.0))),
        ..ScrollWheelEvent::default()
    });

    assert_eq!(background_scrolls.load(AtomicOrdering::Relaxed), 0);
}
