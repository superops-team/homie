use objc2::MainThreadOnly;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSBezelStyle, NSBox, NSBoxType, NSButton, NSColor, NSFocusRingType, NSFont, NSFontWeightMedium,
    NSFontWeightSemibold, NSImage, NSImageView, NSLineBreakMode, NSTextField,
};
use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};

use homie_proto::{AgentKind, AttentionLevel, RiskHint, SessionId, SessionRecord, SessionStatus};

use super::MAX_BODY_ROWS;
use crate::store::StoreSnapshot;

#[derive(Clone, Copy)]
pub(super) enum MenuBodyRow<'a> {
    Project { name: &'a str, count: usize },
    Session(&'a SessionRecord),
}

/// Hashes everything the panel body and header display, so an identical
/// publish never pays NSView teardown.
pub(super) fn panel_fingerprint(
    rows: &[MenuBodyRow<'_>],
    remaining: usize,
    total_sessions: usize,
    selected: Option<&SessionId>,
    global_attention: AttentionLevel,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    for row in rows {
        match row {
            MenuBodyRow::Project { name, count } => {
                0u8.hash(&mut hasher);
                name.hash(&mut hasher);
                count.hash(&mut hasher);
            }
            MenuBodyRow::Session(session) => {
                1u8.hash(&mut hasher);
                session.id.0.hash(&mut hasher);
                display_title(session).hash(&mut hasher);
                std::mem::discriminant(&session.attention()).hash(&mut hasher);
                session.effective_kind().id().hash(&mut hasher);
                session.hibernation.is_some().hash(&mut hasher);
                if let Some(detail) = &session.needs_input {
                    detail.summary.hash(&mut hasher);
                    std::mem::discriminant(&detail.risk_hint).hash(&mut hasher);
                }
            }
        }
    }
    remaining.hash(&mut hasher);
    total_sessions.hash(&mut hasher);
    selected.map(|id| id.0.as_str()).hash(&mut hasher);
    std::mem::discriminant(&global_attention).hash(&mut hasher);
    hasher.finish()
}

pub(super) fn menu_rows(snapshot: &StoreSnapshot) -> Vec<MenuBodyRow<'_>> {
    let mut rows = Vec::new();
    for project in &snapshot.projects {
        let sessions: Vec<_> = snapshot
            .sessions
            .iter()
            .filter(|session| session.project_id == project.id && !session.is_archived())
            .collect();
        if sessions.is_empty() {
            continue;
        }
        rows.push(MenuBodyRow::Project {
            name: &project.name,
            count: sessions.len(),
        });
        for session in sessions {
            rows.push(MenuBodyRow::Session(session.as_ref()));
        }
    }

    if rows.len() > MAX_BODY_ROWS {
        rows.truncate(MAX_BODY_ROWS - 1);
        // Never leave a folder heading stranded at the bottom of the menu.
        if matches!(rows.last(), Some(MenuBodyRow::Project { .. })) {
            rows.pop();
        }
    }
    rows
}

pub(super) fn active_session_count(snapshot: &StoreSnapshot) -> usize {
    snapshot
        .projects
        .iter()
        .map(|project| {
            snapshot
                .sessions
                .iter()
                .filter(|session| session.project_id == project.id && !session.is_archived())
                .count()
        })
        .sum()
}

pub(super) struct TrailingStatus {
    pub(super) text: &'static str,
    pub(super) width: f64,
    pub(super) color: Retained<NSColor>,
}

pub(super) fn trailing_status(session: &SessionRecord) -> Option<TrailingStatus> {
    if session.hibernation.is_some() {
        return Some(TrailingStatus {
            text: "asleep",
            width: 48.0,
            color: NSColor::tertiaryLabelColor(),
        });
    }
    match session.attention() {
        AttentionLevel::NeedsInput => Some(TrailingStatus {
            text: "needs you",
            width: 58.0,
            color: if session
                .needs_input
                .as_ref()
                .is_some_and(|detail| detail.risk_hint == RiskHint::Destructive)
            {
                NSColor::systemRedColor()
            } else {
                NSColor::systemOrangeColor()
            },
        }),
        AttentionLevel::DoneUnseen => Some(TrailingStatus {
            text: "done",
            width: 40.0,
            color: NSColor::systemGreenColor(),
        }),
        _ => None,
    }
}

pub(super) fn agent_symbol(kind: &AgentKind) -> &'static str {
    match kind.id() {
        AgentKind::CLAUDE_CODE_ID => "sparkles",
        AgentKind::CODEX_ID => "circle.hexagongrid.fill",
        AgentKind::CURSOR_ID => "cursorarrow.rays",
        AgentKind::GEMINI_ID => "diamond.fill",
        AgentKind::SHELL_ID | AgentKind::GENERIC_ID => "terminal.fill",
        _ => "circle.fill",
    }
}

pub(super) fn session_color(session: &SessionRecord) -> Retained<NSColor> {
    if session.hibernation.is_some() {
        return NSColor::tertiaryLabelColor();
    }
    match session.attention() {
        AttentionLevel::NeedsInput => {
            if session
                .needs_input
                .as_ref()
                .is_some_and(|detail| detail.risk_hint == RiskHint::Destructive)
            {
                NSColor::systemRedColor()
            } else {
                NSColor::systemOrangeColor()
            }
        }
        AttentionLevel::DoneUnseen => NSColor::systemGreenColor(),
        AttentionLevel::Working => match session.effective_kind().id() {
            AgentKind::CLAUDE_CODE_ID => {
                NSColor::colorWithSRGBRed_green_blue_alpha(0.851, 0.467, 0.341, 1.0)
            }
            AgentKind::GEMINI_ID => {
                NSColor::colorWithSRGBRed_green_blue_alpha(0.306, 0.510, 0.933, 1.0)
            }
            _ => NSColor::labelColor().colorWithAlphaComponent(0.82),
        },
        AttentionLevel::IdleSeen | AttentionLevel::None | AttentionLevel::Unknown => {
            NSColor::tertiaryLabelColor()
        }
    }
}

pub(super) fn display_title(session: &SessionRecord) -> &str {
    if session.title.is_empty() {
        match session.status {
            SessionStatus::Exited(_) => "Ended",
            _ => "Untitled",
        }
    } else {
        &session.title
    }
}

#[derive(Clone, Copy)]
pub(super) enum FontStyle {
    Regular,
    Medium,
    Semibold,
}

pub(super) fn system_font(size: f64, style: FontStyle) -> Retained<NSFont> {
    match style {
        FontStyle::Regular => NSFont::systemFontOfSize(size),
        // SAFETY: AppKit exports these process-lifetime CGFloat constants.
        FontStyle::Medium => NSFont::systemFontOfSize_weight(size, unsafe { NSFontWeightMedium }),
        FontStyle::Semibold => {
            NSFont::systemFontOfSize_weight(size, unsafe { NSFontWeightSemibold })
        }
    }
}

pub(super) fn label(
    text: &str,
    size: f64,
    style: FontStyle,
    color: &NSColor,
    frame: NSRect,
    mtm: MainThreadMarker,
) -> Retained<NSTextField> {
    let label = NSTextField::labelWithString(&NSString::from_str(text), mtm);
    label.setFrame(frame);
    label.setFont(Some(&system_font(size, style)));
    label.setTextColor(Some(color));
    label.setMaximumNumberOfLines(1);
    label.setUsesSingleLineMode(true);
    label.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
    label.setAllowsDefaultTighteningForTruncation(true);
    label
}

pub(super) fn symbol_image(name: &str, fallback: &str) -> Retained<NSImage> {
    let description = NSString::from_str(name);
    let image = NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str(name),
        Some(&description),
    )
    .or_else(|| {
        NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str(fallback),
            Some(&description),
        )
    })
    .expect("macOS 15 always provides the fallback menu symbols");
    image.setTemplate(true);
    image
}

pub(super) fn symbol_view(
    name: &str,
    fallback: &str,
    color: &NSColor,
    frame: NSRect,
    mtm: MainThreadMarker,
) -> Retained<NSImageView> {
    let image = symbol_image(name, fallback);
    let view = NSImageView::imageViewWithImage(&image, mtm);
    view.setFrame(frame);
    view.setContentTintColor(Some(color));
    view
}

pub(super) fn separator(frame: NSRect, mtm: MainThreadMarker) -> Retained<NSBox> {
    let separator = NSBox::initWithFrame(NSBox::alloc(mtm), frame);
    separator.setBoxType(NSBoxType::Separator);
    separator
}

pub(super) fn style_footer_button(button: &NSButton, prominent: bool) {
    button.setFont(Some(&system_font(
        13.0,
        if prominent {
            FontStyle::Medium
        } else {
            FontStyle::Regular
        },
    )));
    button.setBordered(true);
    button.setBezelStyle(if prominent {
        NSBezelStyle::AccessoryBarAction
    } else {
        NSBezelStyle::AccessoryBar
    });
    button.setShowsBorderOnlyWhileMouseInside(!prominent);
    let tint = if prominent {
        NSColor::labelColor()
    } else {
        NSColor::secondaryLabelColor()
    };
    button.setContentTintColor(Some(&tint));
    button.setRefusesFirstResponder(true);
    button.setFocusRingType(NSFocusRingType::None);
}

pub(super) fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect::new(NSPoint::new(x, y), NSSize::new(width, height))
}
