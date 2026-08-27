use std::io::{self, Write};

use crate::rendering::{Buffer, Color, PlotViewport};
use super::{move_cursor, DEFAULT_COLOR, SYNC_BEGIN, SYNC_END};


pub struct TerminalPresenter {
    front: Buffer,
    back: Buffer,
    output: String,
}

impl TerminalPresenter {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            front: Buffer::new(width, height),
            back: Buffer::new(width, height),
            output: String::new(),
        }
    }

    /**
     * Prepare to fill next frame: Clear back buffer 
     * and return as borrowed mutable 
     */
    pub fn begin_frame(&mut self) -> &mut Buffer {
        self.back.clear();
        &mut self.back
    }

    /**
     * Write out back buffer changes to terminal, only
     * sends cursor/write commands for character changes
     */
    pub fn present(
        &mut self,
        out: &mut impl Write,
    ) -> io::Result<()> {
        // Compare front / back buffer for changes
        if self.front == self.back {
            return Ok(());  // no update
        }

        // Clear output string for updates
        self.output.clear();
        self.output.push_str(SYNC_BEGIN);

        // -------------------------
        // Set output cursor move commands / character
        // write commands, then write to stdout
        // -------------------------
        let height = self.back.height();
        let width  = self.back.width();

        let mut active_color = None;

        for y in 0..height {
            let mut x = 0;

            while x < width {
                // Skip unchanged cells
                while x < width && self.front.get(x, y) == self.back.get(x, y) 
                {
                        x += 1
                }

                if x >= width {
                    break;
                }

                // Track consecutive changes
                let run_start = x;

                while x < width && self.front.get(x, y) != self.back.get(x, y)
                {
                    x += 1;
                }

                // Move cursor to x_start (add move command to output string)
                move_cursor(&mut self.output, y, run_start);

                // Write all changed cells in diff run
                for run_x in run_start..x {
                    let cell = self.back.get(run_x, y);
                    
                    // Push character with color
                    if active_color != Some(cell.fg) {
                        use std::fmt::Write as _;

                        match cell.fg {
                            Color::Default => self.output.push_str(DEFAULT_COLOR),
                            Color::Rgb(r, g, b) => {
                                write!(self.output, "\x1b[38;2;{r};{g};{b}m")
                                    .expect("writing to String cannot fail");
                            }
                        }

                        active_color = Some(cell.fg);
                    }

                    self.output.push(cell.ch);
                } 
            }
        }

        self.output.push_str(DEFAULT_COLOR);
        self.output.push_str(SYNC_END);

        // Execute output string cursor commands / character writes
        out.write_all(self.output.as_bytes())?;
        out.flush()?;

        // Swap front / back buffers
        std::mem::swap(&mut self.front, &mut self.back);

        Ok(())
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.front = Buffer::new(width, height);
        self.back = Buffer::new(width, height);
    }
}

// terminal size, else the fallback.
pub fn canvas_dims() -> (usize, usize) {
    let (tw, th) = terminal_size::terminal_size()
        .map(|(w, h)| (w.0 as usize, h.0 as usize))
        .unwrap_or((100, 40));

    (
        tw, 
        th.saturating_sub(1),
    )
}

pub fn canvas_viewport_dims(
    viewport: &PlotViewport,
    cell_aspect: f64,
) -> (usize, usize) {
    let (tw, th) = canvas_dims();

    let x_range = (viewport.x_max - viewport.x_min).abs();
    let y_range = (viewport.y_max - viewport.y_min).abs();

    if x_range <= f64::EPSILON
        || y_range <= f64::EPSILON
        || cell_aspect <= 0.0
    {
        return (tw, th);
    }

    let viewport_aspect = x_range / y_range;

    // Required column-to-row ratio. For a square viewport and cells
    // that are half as wide as tall, we need twice as many columns.
    let target_cell_aspect = viewport_aspect / cell_aspect;
    let available_cell_aspect = tw as f64 / th as f64;

    if available_cell_aspect > target_cell_aspect {
        let height = th;
        let width = (height as f64 * target_cell_aspect).round() as usize;
        (width, height)
    } else {
        let width = tw;
        let height = (width as f64 / target_cell_aspect).round() as usize;
        (width, height)
    }
}