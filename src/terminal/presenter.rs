use std::io::{self, Write};

use crate::rendering::Buffer;
use super::{move_cursor, SYNC_BEGIN, SYNC_END};


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
                    self.output.push(cell.ch);
                } 
            }
        }

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