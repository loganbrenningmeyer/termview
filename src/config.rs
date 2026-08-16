use std::time::Duration;

use termview::canvas::Canvas;

pub struct AppConfig {
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub fps: u32,
    pub degrees_per_second: f64,
    pub spacing: f64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            fps: 100,
            degrees_per_second: 60.0,
            spacing: 1.3,
        }
    }
}

impl AppConfig {
    pub fn frame_time(&self) -> Duration {
        Duration::from_secs_f64(1.0 / self.fps as f64)
    }

    /// Explicit config wins, else terminal size, else the fallback.
    pub fn canvas_dims(&self) -> (usize, usize) {
        let (tw, th) = Canvas::terminal_dims();
        (
            self.width.unwrap_or(tw),
            self.height.unwrap_or(th.saturating_sub(1)),
        )
    }
}

