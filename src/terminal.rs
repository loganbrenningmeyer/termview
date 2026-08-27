mod ansi;
mod presenter;

pub use ansi::{
    ALT_SCREEN_ENTER,
    ALT_SCREEN_LEAVE,
    CLEAR_SCREEN,
    CURSOR_HIDE,
    CURSOR_HOME,
    CURSOR_SHOW,
    DEFAULT_COLOR,
    SYNC_BEGIN,
    SYNC_END,
    enter_fullscreen,
    leave_fullscreen,
    move_cursor,
    present,
};
pub use presenter::{canvas_dims, canvas_viewport_dims, TerminalPresenter};
