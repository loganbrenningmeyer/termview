use crossterm::event::{KeyEvent};

use crate::rendering::{Buffer, Cell};
use super::{PlotController, WaveformController};


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
pub enum ContentController {
    Plot(PlotController),
    Waveform(WaveformController),
}


/**
 * Controller trait for different pane types. Implementations own state
 * and handle rendering, keyboard input, and time-based updates.
 */
pub trait PaneController {
    fn render(&self, target: &mut Buffer);
    fn handle_key(&mut self, key: KeyEvent) -> KeyResult;
    fn update(&mut self, delta_s: f64) -> bool {
        false
    }
}


impl PaneController for ContentController {
    fn render(&self, target: &mut Buffer) {
        match self {
            Self::Plot(controller) => controller.render(target),
            Self::Waveform(controller) => controller.render(target),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        match self {
            Self::Plot(controller) => controller.handle_key(key),
            Self::Waveform(controller) => controller.handle_key(key),
        }
    }

    fn update(&mut self, delta_s: f64) -> bool {
        match self {
            Self::Plot(controller) => controller.update(delta_s),
            Self::Waveform(controller) => controller.update(delta_s),
        }
    }
}


/**
 * Holds and renders typed commands in the bottom pane.
 */
pub struct CommandController {
    pub text: String,
}

impl PaneController for CommandController {
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
