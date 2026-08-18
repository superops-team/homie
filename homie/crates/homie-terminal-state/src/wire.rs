use alacritty_terminal::term::cell::{Cell, Flags};
use alacritty_terminal::vte::ansi::{Color, NamedColor, Rgb};
use homie_proto::grid::{GridCell, TermColor, TermStyle};

/// Maps an alacritty cell to the wire cell the client renders.
///
/// The existing Rust wire vocabulary uses `Default`/`DefaultInverted` for the
/// two default colors, ANSI indices for 0–255, and stable protocol style bits.
pub(crate) fn wire_cell(cell: &alacritty_terminal::term::cell::Cell) -> GridCell {
    let scalar = if cell.c == '\0' { 32 } else { cell.c as u32 };
    GridCell::new(
        scalar,
        wire_color(cell.fg),
        wire_color(cell.bg),
        wire_style(cell.flags),
    )
}

fn wire_color(color: Color) -> TermColor {
    match color {
        Color::Named(NamedColor::Foreground) => TermColor::Default,
        Color::Named(NamedColor::Background) => TermColor::DefaultInverted,
        Color::Named(named) => {
            let index = named as usize;
            if index < 16 {
                TermColor::Ansi(index as u8)
            } else if (NamedColor::DimBlack as usize..=NamedColor::DimWhite as usize)
                .contains(&index)
            {
                // Dim variants render as their base color; DIM rides the style.
                TermColor::Ansi((index - NamedColor::DimBlack as usize) as u8)
            } else {
                TermColor::Default
            }
        }
        Color::Indexed(index) => TermColor::Ansi(index),
        Color::Spec(rgb) => TermColor::Rgb(rgb.r, rgb.g, rgb.b),
    }
}

fn wire_style(flags: Flags) -> TermStyle {
    let mut style = TermStyle::empty();
    if flags.intersects(Flags::BOLD | Flags::DIM_BOLD) {
        style |= TermStyle::BOLD;
    }
    if flags.intersects(Flags::ALL_UNDERLINES) {
        style |= TermStyle::UNDERLINE;
    }
    if flags.contains(Flags::INVERSE) {
        style |= TermStyle::INVERSE;
    }
    if flags.contains(Flags::HIDDEN) {
        style |= TermStyle::INVISIBLE;
    }
    if flags.intersects(Flags::DIM | Flags::DIM_BOLD) {
        style |= TermStyle::DIM;
    }
    if flags.contains(Flags::ITALIC) {
        style |= TermStyle::ITALIC;
    }
    if flags.contains(Flags::STRIKEOUT) {
        style |= TermStyle::CROSSED_OUT;
    }
    style
}

/// The SGR sequence that reproduces a cell's attributes.
pub(crate) fn sgr(cell: &GridCell) -> String {
    let mut codes: Vec<String> = vec!["0".into()];
    if cell.style.contains(TermStyle::BOLD) {
        codes.push("1".into());
    }
    if cell.style.contains(TermStyle::DIM) {
        codes.push("2".into());
    }
    if cell.style.contains(TermStyle::ITALIC) {
        codes.push("3".into());
    }
    if cell.style.contains(TermStyle::UNDERLINE) {
        codes.push("4".into());
    }
    if cell.style.contains(TermStyle::BLINK) {
        codes.push("5".into());
    }
    if cell.style.contains(TermStyle::INVERSE) {
        codes.push("7".into());
    }
    if cell.style.contains(TermStyle::INVISIBLE) {
        codes.push("8".into());
    }
    if cell.style.contains(TermStyle::CROSSED_OUT) {
        codes.push("9".into());
    }
    append_color(cell.fg, true, &mut codes);
    append_color(cell.bg, false, &mut codes);
    format!("\x1b[{}m", codes.join(";"))
}

fn append_color(color: TermColor, foreground: bool, codes: &mut Vec<String>) {
    match color {
        TermColor::Default | TermColor::DefaultInverted => {
            codes.push(if foreground { "39" } else { "49" }.into());
        }
        TermColor::Ansi(value) => {
            codes.push(if foreground { "38" } else { "48" }.into());
            codes.push("5".into());
            codes.push(value.to_string());
        }
        TermColor::Rgb(red, green, blue) => {
            codes.push(if foreground { "38" } else { "48" }.into());
            codes.push("2".into());
            codes.push(red.to_string());
            codes.push(green.to_string());
            codes.push(blue.to_string());
        }
    }
}

pub(crate) fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

pub(crate) fn emulator_cell(cell: GridCell) -> Cell {
    let mut flags = Flags::empty();
    if cell.style.contains(TermStyle::BOLD) {
        flags.insert(Flags::BOLD);
    }
    if cell.style.contains(TermStyle::UNDERLINE) {
        flags.insert(Flags::UNDERLINE);
    }
    if cell.style.contains(TermStyle::INVERSE) {
        flags.insert(Flags::INVERSE);
    }
    if cell.style.contains(TermStyle::INVISIBLE) {
        flags.insert(Flags::HIDDEN);
    }
    if cell.style.contains(TermStyle::DIM) {
        flags.insert(Flags::DIM);
    }
    if cell.style.contains(TermStyle::ITALIC) {
        flags.insert(Flags::ITALIC);
    }
    if cell.style.contains(TermStyle::CROSSED_OUT) {
        flags.insert(Flags::STRIKEOUT);
    }
    Cell {
        c: char::from_u32(cell.scalar)
            .filter(|_| cell.scalar != 0)
            .unwrap_or(' '),
        fg: emulator_color(cell.fg),
        bg: emulator_color(cell.bg),
        flags,
        extra: None,
    }
}

fn emulator_color(color: TermColor) -> Color {
    match color {
        TermColor::Default => Color::Named(NamedColor::Foreground),
        TermColor::DefaultInverted => Color::Named(NamedColor::Background),
        TermColor::Ansi(index) => Color::Indexed(index),
        TermColor::Rgb(r, g, b) => Color::Spec(Rgb { r, g, b }),
    }
}
