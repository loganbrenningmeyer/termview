// termview/src/term.rs
//! ANSI/VT100 terminal control sequences.
//!
//! `\x1b` is ESC; `ESC [` is the Control Sequence Introducer (CSI).
//! A `?` marks a DEC private mode, where `h` sets and `l` resets it.

use std::io::{self, Write};

// Erase the entire screen. Does not move the cursor.
pub const CLEAR_SCREEN: &str = "\x1b[2J";

// Move the cursor to row 1, column 1.
pub const CURSOR_HOME: &str = "\x1b[H";

pub const CURSOR_HIDE: &str = "\x1b[?25l";
pub const CURSOR_SHOW: &str = "\x1b[?25h";

// Alternate screen buffer: a separate non-scrolling surface. Leaving it
// restores whatever the terminal was showing before.
pub const ALT_SCREEN_ENTER: &str = "\x1b[?1049h";
pub const ALT_SCREEN_LEAVE: &str = "\x1b[?1049l";

// Synchronized output: the terminal holds its repaint until SYNC_END,
// so a frame is never shown half-drawn. Ignored by terminals that
// don't implement it.
pub const SYNC_BEGIN: &str = "\x1b[?2026h";
pub const SYNC_END: &str = "\x1b[?2026l";

// Take over the screen: alternate buffer, no cursor, cleared.
pub fn enter_fullscreen(out: &mut impl Write) -> io::Result<()> {
    write!(out, "{ALT_SCREEN_ENTER}{CURSOR_HIDE}{CLEAR_SCREEN}")?;
    out.flush()
}

// Hand the screen back. Must run before exit or the user's shell is
// left cursorless on the alternate buffer.
pub fn leave_fullscreen(out: &mut impl Write) -> io::Result<()> {
    write!(out, "{CURSOR_SHOW}{ALT_SCREEN_LEAVE}")?;
    out.flush()
}

// Repaint the screen from the top-left with `frame`, as one atomic update.
pub fn present(out: &mut impl Write, frame: &str) -> io::Result<()> {
    write!(out, "{SYNC_BEGIN}{CURSOR_HOME}")?;
    out.write_all(frame.as_bytes())?;
    write!(out, "{SYNC_END}")?;
    out.flush()
}

// Move cursor to position at 0-indexed (row, col), 
// where ANSI coords are 1-indexed (row + 1, col + 1)
pub fn move_cursor(output: &mut String, row: usize, col: usize) {
    std::fmt::Write::write_fmt(
        output,
        format_args!("\x1b[{};{}H", row + 1, col + 1),
    )
    .expect("writing to a String cannot fail");
}