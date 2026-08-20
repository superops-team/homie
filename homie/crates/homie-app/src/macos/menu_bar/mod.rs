//! Native menu bar surface using the same visual grammar as the GPUI sidebar.

use std::sync::{Arc, RwLock};

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadOnly, msg_send, sel};
use objc2_app_kit::{
    NSAutoresizingMaskOptions, NSBackingStoreType, NSBox, NSButton, NSCellImagePosition, NSColor,
    NSFloatingWindowLevel, NSImage, NSImageView, NSPanel, NSSquareStatusItemLength, NSStatusBar,
    NSStatusBarButton, NSStatusItem, NSTextAlignment, NSTextField, NSView,
    NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView,
    NSWindowAnimationBehavior, NSWindowStyleMask,
};
use objc2_foundation::{MainThreadMarker, NSPoint, NSSize, NSString};

use homie_proto::{AttentionLevel, SessionId};

use crate::store::{SessionStore, StoreSnapshot};

use model::{
    FontStyle, MenuBodyRow, active_session_count, label, menu_rows, panel_fingerprint, rect,
    separator, style_footer_button, symbol_image, symbol_view,
};
use target::MenuBarTarget;

mod model;
mod render;
mod target;

const POPUP_WIDTH: f64 = 344.0;
const HEADER_HEIGHT: f64 = 46.0;
const FOOTER_HEIGHT: f64 = 44.0;
const ROW_HEIGHT: f64 = 28.0;
const BODY_PADDING: f64 = 6.0;
const EMPTY_BODY_HEIGHT: f64 = 92.0;
const MAX_BODY_ROWS: usize = 11;

pub struct NativeMenuBar {
    _status_item: Retained<NSStatusItem>,
    button: Retained<NSStatusBarButton>,
    panel: Retained<NSPanel>,
    surface: Retained<NSVisualEffectView>,
    header_icon: Retained<NSImageView>,
    title_label: Retained<NSTextField>,
    activity_label: Retained<NSTextField>,
    header_divider: Retained<NSBox>,
    body: Retained<NSView>,
    // NSControl target is not retained.
    _target: Retained<MenuBarTarget>,
    /// Hash of the last-rendered panel content. Rebuilding tears down and
    /// recreates every NSView in the body, so identical publishes are skipped.
    last_fingerprint: Option<u64>,
}

impl NativeMenuBar {
    #[must_use]
    pub fn new(mtm: MainThreadMarker, store: Arc<RwLock<SessionStore>>) -> Option<Self> {
        let status_item =
            NSStatusBar::systemStatusBar().statusItemWithLength(NSSquareStatusItemLength);
        let button = status_item.button(mtm)?;
        button.setToolTip(Some(&NSString::from_str("homie")));

        let panel = NSPanel::initWithContentRect_styleMask_backing_defer(
            NSPanel::alloc(mtm),
            rect(0.0, 0.0, POPUP_WIDTH, 140.0),
            NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel,
            NSBackingStoreType::Buffered,
            false,
        );
        panel.setFloatingPanel(true);
        panel.setLevel(NSFloatingWindowLevel);
        panel.setHasShadow(true);
        panel.setHidesOnDeactivate(true);
        panel.setOpaque(false);
        panel.setBackgroundColor(Some(&NSColor::clearColor()));
        panel.setAnimationBehavior(NSWindowAnimationBehavior::UtilityWindow);

        // A real sidebar material is the strongest connection to the main
        // window. The old opaque NSTextField panel looked like a debug dump.
        let surface = NSVisualEffectView::initWithFrame(
            NSVisualEffectView::alloc(mtm),
            rect(0.0, 0.0, POPUP_WIDTH, 140.0),
        );
        surface.setMaterial(NSVisualEffectMaterial::Sidebar);
        surface.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
        surface.setState(NSVisualEffectState::Active);
        surface.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        surface.setWantsLayer(true);
        // NSView exposes CALayer only when objc2-quartz-core is linked. Keep
        // this tiny styling call dynamic so the AppKit surface does not pull a
        // second graphics binding into the app just for two properties.
        unsafe {
            let layer: *mut AnyObject = msg_send![&*surface, layer];
            if !layer.is_null() {
                let _: () = msg_send![layer, setCornerRadius: 12.0_f64];
                let _: () = msg_send![layer, setMasksToBounds: true];
            }
        }
        panel.setContentView(Some(&surface));

        let header_icon = symbol_view(
            "waveform.circle",
            "waveform",
            &NSColor::secondaryLabelColor(),
            rect(14.0, 0.0, 20.0, 20.0),
            mtm,
        );
        surface.addSubview(&header_icon);

        let title_label = label(
            "homie",
            13.0,
            FontStyle::Semibold,
            &NSColor::labelColor(),
            rect(42.0, 0.0, 80.0, 18.0),
            mtm,
        );
        surface.addSubview(&title_label);

        let activity_label = label(
            "No active sessions",
            11.0,
            FontStyle::Medium,
            &NSColor::secondaryLabelColor(),
            rect(148.0, 0.0, POPUP_WIDTH - 162.0, 18.0),
            mtm,
        );
        activity_label.setAlignment(NSTextAlignment::Right);
        surface.addSubview(&activity_label);

        let header_divider = separator(rect(10.0, 0.0, POPUP_WIDTH - 20.0, 1.0), mtm);
        surface.addSubview(&header_divider);

        let body = NSView::initWithFrame(
            NSView::alloc(mtm),
            rect(0.0, FOOTER_HEIGHT, POPUP_WIDTH, EMPTY_BODY_HEIGHT),
        );
        surface.addSubview(&body);

        let footer_divider = separator(rect(10.0, FOOTER_HEIGHT, POPUP_WIDTH - 20.0, 1.0), mtm);
        surface.addSubview(&footer_divider);

        let target = MenuBarTarget::new(mtm, panel.clone(), button.clone(), store);
        let open_button = unsafe {
            NSButton::buttonWithTitle_target_action(
                &NSString::from_str("Open homie"),
                Some(&*target as &AnyObject),
                Some(sel!(openHomie:)),
                mtm,
            )
        };
        style_footer_button(&open_button, true);
        open_button.setImage(Some(&symbol_image("arrow.up.forward.app", "macwindow")));
        open_button.setImagePosition(NSCellImagePosition::ImageLeading);
        open_button.setImageHugsTitle(true);
        open_button.setFrame(rect(8.0, 8.0, 112.0, 28.0));
        surface.addSubview(&open_button);

        let quit_button = unsafe {
            NSButton::buttonWithTitle_target_action(
                &NSString::from_str("Quit"),
                Some(&*target as &AnyObject),
                Some(sel!(quitHomie:)),
                mtm,
            )
        };
        style_footer_button(&quit_button, false);
        quit_button.setFrame(rect(POPUP_WIDTH - 66.0, 8.0, 58.0, 28.0));
        surface.addSubview(&quit_button);

        unsafe {
            button.setTarget(Some(&*target as &AnyObject));
            button.setAction(Some(sel!(toggleHomieMenu:)));
        }

        let mut menu_bar = Self {
            _status_item: status_item,
            button,
            panel,
            surface,
            header_icon,
            title_label,
            activity_label,
            header_divider,
            body,
            _target: target,
            last_fingerprint: None,
        };
        menu_bar.set_attention(AttentionLevel::None);
        Some(menu_bar)
    }

    pub fn update(&mut self, snapshot: &StoreSnapshot) {
        self.set_attention(snapshot.global_attention);
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };

        // A hidden panel needs only the status-item glyph above. Opening the
        // panel requests a fresh snapshot publish (see `toggle_menu`), which
        // re-enters here with the panel visible.
        if !self.panel.isVisible() {
            self.last_fingerprint = None;
            return;
        }

        let rows = menu_rows(snapshot);
        let visible_sessions = rows
            .iter()
            .filter(|row| matches!(row, MenuBodyRow::Session(_)))
            .count();
        let total_sessions = active_session_count(snapshot);
        let remaining = total_sessions.saturating_sub(visible_sessions);
        let fingerprint = panel_fingerprint(
            &rows,
            remaining,
            total_sessions,
            snapshot.selected_session_id.as_ref(),
            snapshot.global_attention,
        );
        if self.last_fingerprint == Some(fingerprint) {
            return;
        }
        self.last_fingerprint = Some(fingerprint);
        let body_height = if rows.is_empty() {
            EMPTY_BODY_HEIGHT
        } else {
            BODY_PADDING * 2.0
                + (rows.len() as f64 + usize::from(remaining > 0) as f64) * ROW_HEIGHT
        };
        let height = HEADER_HEIGHT + body_height + FOOTER_HEIGHT;

        let old_top_left = self.panel.isVisible().then(|| {
            let frame = self.panel.frame();
            NSPoint::new(frame.origin.x, frame.origin.y + frame.size.height)
        });
        self.panel.setContentSize(NSSize::new(POPUP_WIDTH, height));
        if let Some(top_left) = old_top_left {
            self.panel.setFrameTopLeftPoint(top_left);
        }
        self.surface.setFrame(rect(0.0, 0.0, POPUP_WIDTH, height));

        let header_y = FOOTER_HEIGHT + body_height;
        self.header_icon
            .setFrame(rect(14.0, header_y + 13.0, 20.0, 20.0));
        self.title_label
            .setFrame(rect(42.0, header_y + 14.0, 80.0, 18.0));
        self.activity_label
            .setFrame(rect(148.0, header_y + 14.0, POPUP_WIDTH - 162.0, 18.0));
        self.header_divider
            .setFrame(rect(10.0, header_y, POPUP_WIDTH - 20.0, 1.0));
        self.body
            .setFrame(rect(0.0, FOOTER_HEIGHT, POPUP_WIDTH, body_height));

        self.update_activity(snapshot, total_sessions);
        self.rebuild_body(
            &rows,
            remaining,
            body_height,
            snapshot.selected_session_id.as_ref(),
            mtm,
        );
    }

    fn set_attention(&mut self, attention: AttentionLevel) {
        let symbol = match attention {
            AttentionLevel::NeedsInput => "waveform.circle.fill",
            AttentionLevel::DoneUnseen => "waveform.circle.fill",
            AttentionLevel::Working => "waveform.circle",
            AttentionLevel::IdleSeen | AttentionLevel::None | AttentionLevel::Unknown => "waveform",
        };
        if let Some(image) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str(symbol),
            Some(&NSString::from_str("homie agent status")),
        ) {
            image.setTemplate(true);
            self.button.setImage(Some(&image));
        }

        let (symbol, fallback, color) = match attention {
            AttentionLevel::NeedsInput => (
                "waveform.circle.fill",
                "exclamationmark.circle.fill",
                NSColor::systemOrangeColor(),
            ),
            AttentionLevel::DoneUnseen => (
                "checkmark.circle.fill",
                "waveform.circle.fill",
                NSColor::systemGreenColor(),
            ),
            AttentionLevel::Working => (
                "waveform.circle",
                "waveform",
                NSColor::secondaryLabelColor(),
            ),
            AttentionLevel::IdleSeen | AttentionLevel::None | AttentionLevel::Unknown => {
                ("waveform", "circle", NSColor::tertiaryLabelColor())
            }
        };
        self.header_icon
            .setImage(Some(&symbol_image(symbol, fallback)));
        self.header_icon.setContentTintColor(Some(&color));
    }

    fn update_activity(&self, snapshot: &StoreSnapshot, total_sessions: usize) {
        let needs_input = snapshot
            .sessions
            .iter()
            .filter(|session| !session.is_archived())
            .filter(|session| session.attention() == AttentionLevel::NeedsInput)
            .count();
        let done = snapshot
            .sessions
            .iter()
            .filter(|session| !session.is_archived())
            .filter(|session| session.attention() == AttentionLevel::DoneUnseen)
            .count();
        let (text, color) = if needs_input > 0 {
            (
                format!("{needs_input} need you"),
                NSColor::systemOrangeColor(),
            )
        } else if done > 0 {
            (format!("{done} finished"), NSColor::systemGreenColor())
        } else if total_sessions == 0 {
            (
                "No active sessions".to_owned(),
                NSColor::secondaryLabelColor(),
            )
        } else {
            (
                format!(
                    "{total_sessions} session{}",
                    if total_sessions == 1 { "" } else { "s" }
                ),
                NSColor::secondaryLabelColor(),
            )
        };
        self.activity_label
            .setStringValue(&NSString::from_str(&text));
        self.activity_label.setTextColor(Some(&color));
    }

    fn rebuild_body(
        &self,
        rows: &[MenuBodyRow<'_>],
        remaining: usize,
        body_height: f64,
        selected_session_id: Option<&SessionId>,
        mtm: MainThreadMarker,
    ) {
        for child in self.body.subviews().iter() {
            child.removeFromSuperview();
        }

        if rows.is_empty() {
            self._target.set_session_ids(Vec::new());
            self.add_empty_state(mtm);
            return;
        }

        let session_ids = rows
            .iter()
            .filter_map(|row| match row {
                MenuBodyRow::Session(session) => Some(session.id.clone()),
                MenuBodyRow::Project { .. } => None,
            })
            .collect();
        self._target.set_session_ids(session_ids);

        let mut y = body_height - BODY_PADDING;
        let mut session_tag = 0;
        for row in rows {
            y -= ROW_HEIGHT;
            match row {
                MenuBodyRow::Project { name, count } => {
                    self.add_project_header(name, *count, y, mtm);
                }
                MenuBodyRow::Session(session) => {
                    self.add_session_row(
                        session,
                        session_tag,
                        selected_session_id == Some(&session.id),
                        y,
                        mtm,
                    );
                    session_tag += 1;
                }
            }
        }
        if remaining > 0 {
            y -= ROW_HEIGHT;
            self.add_more_row(remaining, y, mtm);
        }
    }
}
