use crossterm::event::{KeyEvent};

use super::{
    Animation,
    KeyResult, 
    PlotWidget2d, 
    PlotWidget3d,
    Rect, 
    Widget, 
};
use crate::rendering::{draw_border, draw_text, Buffer, Color};


/**
 * 
 */
#[derive(Debug, PartialEq)]
pub enum InteractionMode {
    Static,
    Interactive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusState {
    Inactive,
    Active,
}

impl FocusState {
    pub fn border_color(self) -> Color {
        match self {
            Self::Inactive => Color::Rgb(160, 160, 160),
            Self::Active => Color::Rgb(160, 160, 0),
        }
    }
}

pub struct Pane<W: Widget> {
    pub title: String,
    pub area: Rect,
    pub buffer: Buffer,
    pub mode: InteractionMode,
    pub focus: FocusState,
    pub widget: W,
}

impl<W: Widget> Pane<W> {
    pub fn new(area: Rect, mode: InteractionMode, focus: FocusState, widget: W) -> Self {
        Self {
            title: String::new(),
            area,
            buffer: Buffer::new(area.width, area.height),
            mode,
            focus,
            widget,
        }
    }

    /**
     * Handle key event depending on Pane's Widget 
     * - e.g., 2D vs 3D commands
     */
    pub fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        match self.mode {
            InteractionMode::Static => KeyResult::Ignored,
            InteractionMode::Interactive => {
                self.widget.handle_key(key)
            }
        }
    }

    /**
     * Render into Buffer using Widget.render() function,
     * then place the buffer into the target frame at the
     * Pane's placement Rect
     */
    pub fn render_into(&mut self, frame: &mut Buffer) {
        self.buffer.clear();

        if self.area.width == 0 || self.area.height == 0 {
            return;
        }

        self.widget.render(&mut self.buffer);

        // Draw border around pane
        draw_border(&mut self.buffer, self.focus.border_color());

        // Write title at top-left of pane
        // - Drawn at (1, 0) pane-local coordinates,
        //   buffer.blit() handles the screen offset afterward
        draw_text(
            &mut self.buffer,
            2,
            0,
            &self.title,
            Color::Rgb(255, 255, 255),
        );

        frame.blit(&mut self.buffer, self.area.x, self.area.y);
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    pub fn set_area(&mut self, area: Rect) {
        if self.buffer.width() != area.width || self.buffer.height() != area.height 
        {
            self.buffer = Buffer::new(area.width, area.height);
        }
        self.area = area;
    }

    pub fn set_focus(&mut self, focus: FocusState) {
        self.focus = focus;
    }

    pub fn set_widget(&mut self, widget: W) {
        self.widget = widget;
    }
}

pub enum PaneContent {
    Plot2d(PlotWidget2d),
    Plot3d(PlotWidget3d),
    Empty,
}

impl PaneContent {
    /**
     * Give a mutable reference to the Animation stored
     * inside the Widget, None if empty pane or static plot
     */
    pub fn animation_mut(&mut self) -> Option<&mut Animation> {
        match self {
            Self::Plot2d(widget) => widget.animation.as_mut(),
            Self::Plot3d(widget) => widget.animation.as_mut(),
            Self::Empty => None,
        }
    }

    /**
     * Sample new values for the current Animation state
     * if Animation is enabled
     */
    pub fn resample(&mut self) {
        match self {
            Self::Plot2d(widget) => widget.resample(),
            Self::Plot3d(widget) => widget.resample(),
            Self::Empty => {}
        }
    }
}

impl Widget for PaneContent {
    fn render(&self, target: &mut Buffer) {
        match self {
            Self::Plot2d(widget) => widget.render(target),
            Self::Plot3d(widget) => widget.render(target),
            Self::Empty => {}
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        match self {
            Self::Plot2d(widget) => widget.handle_key(key),
            Self::Plot3d(widget) => widget.handle_key(key),
            Self::Empty => KeyResult::Ignored,
        }
    }

    fn update(&mut self, delta_s: f64) -> bool {
        match self {
            Self::Plot2d(widget) => widget.update(delta_s),
            Self::Plot3d(widget) => widget.update(delta_s),
            Self::Empty => false
        }
    }
}