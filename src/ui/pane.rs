use crossterm::event::{KeyCode, KeyEvent};


use super::{
    KeyResult, 
    Rect, 
    PaneController, 
};
use crate::{
    rendering::{
        draw_border, draw_text, draw_text_block,
        Buffer, 
        Color,
    },
};


/**
 * Self-contained section of the terminal screen, owning its
 * own buffer, taking up a specified area, and containing a
 * PaneController that provides its rendering and key handling
 */
pub struct Pane<C: PaneController> {
    pub title: String,
    pub area: Rect,
    pub buffer: Buffer,
    pub mode: InteractionMode,
    pub focus: FocusState,
    pub show_config: bool,
    pub controller: C,
}

impl<C: PaneController> Pane<C> {
    pub fn new(area: Rect, mode: InteractionMode, focus: FocusState, controller: C) -> Self {
        Self {
            title: String::new(),
            area,
            buffer: Buffer::new(area.width, area.height),
            mode,
            focus,
            show_config: false,
            controller,
        }
    }

    /**
     * Handle key events through the pane's controller
     * - e.g., 2D vs 3D commands
     */
    pub fn handle_key(&mut self, key: KeyEvent) -> KeyResult {
        // Show config
        if key.code == KeyCode::Char('c') {
            self.show_config = !self.show_config;
            return KeyResult::Changed;
        }

        match self.mode {
            InteractionMode::Static => KeyResult::Ignored,
            InteractionMode::Interactive => {
                self.controller.handle_key(key)
            }
        }
    }

    /**
     * Render into Buffer using PaneController.render() function,
     * then place the buffer into the target frame at the
     * Pane's placement Rect
     */
    pub fn render_into(&mut self, frame: &mut Buffer) {
        self.buffer.clear();

        if self.area.width == 0 || self.area.height == 0 {
            return;
        }

        self.controller.render(&mut self.buffer);

        // Draw border around pane
        draw_border(&mut self.buffer, self.focus.border_color(), true);

        // Write title at top-left of pane
        // - Drawn at (1, 0) pane-local coordinates,
        //   buffer.blit() handles the screen offset afterward
        draw_text(
            &mut self.buffer,
            2,
            1,
            &self.title,
            Color::Rgb(255, 255, 255),
            true,
        );

        // Show config at the top-right of pane
        if self.show_config {
            let text = self.controller.config_text();
            
            if !text.is_empty() {
                if let Some(text_width) = text
                    .iter()
                    .map(|line| line.chars().count())
                    .max()
                {
                    let pane_width = self.buffer.width();
                    
                    draw_text_block(
                        &mut self.buffer, 
                        (pane_width - text_width - 4) as isize,
                        1,
                        text,
                        Color::Rgb(255, 255, 255),
                        true,
                        "Current settings",
                    );
                }
            }
        }
            
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

    pub fn set_controller(&mut self, controller: C) {
        self.controller = controller;
    }
}


#[derive(Debug, PartialEq)]
pub enum InteractionMode {
    Static,
    Interactive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusState {
    InactivePane,
    ActivePane,
    InactiveCommand,
    ActiveCommand,
}

impl FocusState {
    pub fn border_color(self) -> Color {
        match self {
            Self::InactivePane => Color::Rgb(120, 120, 120),
            Self::ActivePane => Color::Rgb(120, 120, 200),
            Self::InactiveCommand => Color::Rgb(120, 120, 120),
            Self::ActiveCommand => Color::Rgb(120, 120, 200),
        }
    }
}