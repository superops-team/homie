use objc2::runtime::AnyObject;
use objc2::{msg_send, sel};
use objc2_app_kit::{
    NSBezelStyle, NSButton, NSCellImagePosition, NSColor, NSFocusRingType, NSTextAlignment,
};
use objc2_foundation::{MainThreadMarker, NSString};

use homie_proto::SessionRecord;

use super::model::{
    FontStyle, agent_symbol, display_title, label, rect, session_color, symbol_image, symbol_view,
    system_font, trailing_status,
};
use super::{POPUP_WIDTH, ROW_HEIGHT};

impl super::NativeMenuBar {
    pub(super) fn add_project_header(
        &self,
        name: &str,
        count: usize,
        y: f64,
        mtm: MainThreadMarker,
    ) {
        let icon = symbol_view(
            "folder.fill",
            "folder",
            &NSColor::secondaryLabelColor(),
            rect(16.0, y + 7.0, 14.0, 14.0),
            mtm,
        );
        self.body.addSubview(&icon);

        let name = label(
            name,
            13.0,
            FontStyle::Medium,
            &NSColor::labelColor().colorWithAlphaComponent(0.90),
            rect(38.0, y + 5.0, POPUP_WIDTH - 82.0, 18.0),
            mtm,
        );
        self.body.addSubview(&name);

        let count = label(
            &count.to_string(),
            11.0,
            FontStyle::Medium,
            &NSColor::tertiaryLabelColor(),
            rect(POPUP_WIDTH - 38.0, y + 5.0, 22.0, 18.0),
            mtm,
        );
        count.setAlignment(NSTextAlignment::Right);
        self.body.addSubview(&count);
    }

    pub(super) fn add_session_row(
        &self,
        session: &SessionRecord,
        tag: isize,
        selected: bool,
        y: f64,
        mtm: MainThreadMarker,
    ) {
        let row = unsafe {
            NSButton::buttonWithTitle_target_action(
                &NSString::new(),
                Some(&*self._target as &AnyObject),
                Some(sel!(selectSession:)),
                mtm,
            )
        };
        row.setTag(tag);
        row.setFrame(rect(8.0, y + 1.0, POPUP_WIDTH - 16.0, ROW_HEIGHT - 2.0));
        row.setBordered(true);
        row.setBezelStyle(NSBezelStyle::AccessoryBar);
        row.setShowsBorderOnlyWhileMouseInside(!selected);
        row.setBezelColor(Some(
            &NSColor::labelColor().colorWithAlphaComponent(if selected { 0.10 } else { 0.08 }),
        ));
        row.setRefusesFirstResponder(true);
        row.setFocusRingType(NSFocusRingType::None);
        let tooltip = session.needs_input.as_ref().map_or_else(
            || display_title(session).to_owned(),
            |detail| detail.summary.clone(),
        );
        row.setToolTip(Some(&NSString::from_str(&tooltip)));
        unsafe {
            let accessibility_label = NSString::from_str(display_title(session));
            let _: () = msg_send![&*row, setAccessibilityLabel: &*accessibility_label];
        }

        let icon_color = session_color(session);
        let icon = symbol_view(
            agent_symbol(session.effective_kind()),
            "circle.fill",
            &icon_color,
            rect(18.0, 6.0, 14.0, 14.0),
            mtm,
        );
        row.addSubview(&icon);

        let status = trailing_status(session);
        let status_width = status.as_ref().map_or(0.0, |status| status.width + 8.0);
        let title_color = if session.hibernation.is_some() {
            NSColor::secondaryLabelColor()
        } else {
            NSColor::labelColor().colorWithAlphaComponent(0.82)
        };
        let title = label(
            display_title(session),
            13.0,
            FontStyle::Regular,
            &title_color,
            rect(
                42.0,
                4.0,
                POPUP_WIDTH - 16.0 - 42.0 - 10.0 - status_width,
                18.0,
            ),
            mtm,
        );
        row.addSubview(&title);

        if let Some(status) = status {
            let status_label = label(
                status.text,
                11.0,
                FontStyle::Medium,
                &status.color,
                rect(POPUP_WIDTH - 26.0 - status.width, 4.0, status.width, 18.0),
                mtm,
            );
            status_label.setAlignment(NSTextAlignment::Right);
            row.addSubview(&status_label);
        }
        self.body.addSubview(&row);
    }

    pub(super) fn add_more_row(&self, remaining: usize, y: f64, mtm: MainThreadMarker) {
        let button = unsafe {
            NSButton::buttonWithTitle_target_action(
                &NSString::from_str(&format!(
                    "{remaining} more session{} in homie",
                    if remaining == 1 { "" } else { "s" }
                )),
                Some(&*self._target as &AnyObject),
                Some(sel!(openHomie:)),
                mtm,
            )
        };
        button.setFrame(rect(20.0, y + 1.0, POPUP_WIDTH - 40.0, ROW_HEIGHT - 2.0));
        button.setBordered(true);
        button.setBezelStyle(NSBezelStyle::AccessoryBar);
        button.setShowsBorderOnlyWhileMouseInside(true);
        button.setFont(Some(&system_font(11.0, FontStyle::Medium)));
        button.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
        button.setImage(Some(&symbol_image("ellipsis", "circle.grid.3x3.fill")));
        button.setImagePosition(NSCellImagePosition::ImageLeading);
        button.setImageHugsTitle(true);
        button.setRefusesFirstResponder(true);
        button.setFocusRingType(NSFocusRingType::None);
        self.body.addSubview(&button);
    }

    pub(super) fn add_empty_state(&self, mtm: MainThreadMarker) {
        let icon = symbol_view(
            "waveform",
            "sparkles",
            &NSColor::tertiaryLabelColor(),
            rect((POPUP_WIDTH - 22.0) / 2.0, 52.0, 22.0, 22.0),
            mtm,
        );
        self.body.addSubview(&icon);

        let title = label(
            "No active sessions",
            13.0,
            FontStyle::Medium,
            &NSColor::secondaryLabelColor(),
            rect(32.0, 29.0, POPUP_WIDTH - 64.0, 18.0),
            mtm,
        );
        title.setAlignment(NSTextAlignment::Center);
        self.body.addSubview(&title);

        let hint = label(
            "Open homie to start an agent",
            11.0,
            FontStyle::Regular,
            &NSColor::tertiaryLabelColor(),
            rect(32.0, 11.0, POPUP_WIDTH - 64.0, 16.0),
            mtm,
        );
        hint.setAlignment(NSTextAlignment::Center);
        self.body.addSubview(&hint);
    }
}
