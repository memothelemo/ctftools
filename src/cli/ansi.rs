use anstyle::{AnsiColor, Color, Style};

pub const GRAY: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightBlack)));
pub const BOLD: Style = Style::new().bold();

pub const GREEN: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightGreen)));
pub const GREEN_BOLD: Style = GREEN.bold();

pub const YELLOW: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::BrightYellow)));
pub const YELLOW_BOLD: Style = YELLOW.bold();

pub const RED: Style = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red)));
