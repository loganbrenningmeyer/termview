use crossterm::event::{KeyEvent};

use crate::rendering::{Buffer, Cell};


#[derive(Debug, PartialEq)]
pub enum KeyResult {
    Ignored,
    Changed,
    Exit,
}


/**
 * Animation state for expressions using t
 * which changes over time
 */
pub struct Animation {
    pub time: f64,      // value passed as t
    pub speed: f64,     // speed of animation (scales delta_t)
    pub phase: f64,     // position along the full round trip
    pub period: f64,    // time for one way of the trip
    pub playing: bool,  
}


/**
 * 
 */
pub trait Widget {
    fn render(&self, target: &mut Buffer);
    fn handle_key(&mut self, key: KeyEvent) -> KeyResult;
    fn update(&mut self, _delta_s: f64) -> bool {
        false
    }
}


/**
 * Widget for rendering typed commands in bottom Pane
 */
pub struct CommandWidget {
    pub text: String,
}

impl Widget for CommandWidget {
    fn render(&self, target: &mut Buffer) {
        // Leave room for the pane's border
        if target.width() < 3 || target.height() < 3 {
            return;
        }

        // Border on both sides = width - 2
        let width = target.width() - 2;
        let line = self.text.lines().next().unwrap_or("");  // single line

        // Show the end of long commands while typing
        let skip = line.chars().count().saturating_sub(width);

        // Skip `skip` chars, take up to width chars, and enumerate
        for (x, ch) in line.chars().skip(skip).take(width).enumerate() {
            target.set(
                (x + 1) as isize,
                1,
                Cell::new(ch),
            )
        }
    }

    fn handle_key(&mut self, _: KeyEvent) -> KeyResult {
        KeyResult::Ignored
    }
}