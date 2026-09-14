use crossterm::event::{KeyCode, KeyEvent};


use super::{
    KeyResult, 
    Rect, 
    PaneController, 
};
use crate::{
    rendering::{
        Buffer, Color, draw_border, draw_text, draw_text_block,
    }, 
    repl::HORIZONTAL,
    ui::FocusState::LastActivePane,
};


/**
 * Self-contained section of the terminal screen, owning its
 * own buffer, taking up a specified area, and containing a
 * PaneController that provides its rendering and key handling
 */
pub struct Pane<C: PaneController> {
    pub controller: C,
    pub title: String,
    pub area: Rect,
    pub buffer: Buffer,
    pub mode: InteractionMode,
    pub focus: FocusState,
    pub show_config: bool,
    pub number: Option<usize>,
}

impl<C: PaneController> Pane<C> {
    pub fn new(
        controller: C, 
        area: Rect, 
        mode: InteractionMode, 
        focus: FocusState,
    ) -> Self {
        Self {
            controller,
            title: String::new(),
            area,
            buffer: Buffer::new(area.width, area.height),
            mode,
            focus,
            show_config: false,
            number: None,
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

        // -------------------------
        // Pane title (top-right)
        // -------------------------
        let title_x = self.buffer.width()
            .saturating_sub(self.title.chars().count())
            .saturating_sub(4) as isize;

        draw_text(
            &mut self.buffer,
            title_x,
            1,
            &self.title,
            Color::WHITE,
            true,
        );

        // -------------------------
        // Pane tag (top-right)
        // -------------------------
        let tag = self.controller.tag_text();

        if !tag.is_empty() {
            let tag_padded = format!(" {tag} ");

            let x = self.buffer.width()
                .saturating_sub(tag_padded.chars().count())
                .saturating_sub(2) as isize;

            draw_text(
                &mut self.buffer,
                x,
                0,
                &tag_padded,
                Color::WHITE,
                false,
            );
        }

        // -------------------------
        // Pane number - label (top-left)
        // -------------------------
        if let Some(number) = self.number {
            let label = self.controller.label_text();

            let parts = [
                (format!(" [{number}] "), self.focus.label_color()),
                (HORIZONTAL.to_string(), Color::GRAY),
                (format!(" {} ", label.trim()), Color::WHITE),
            ];

            let mut x = 2;

            for (text, color) in parts {
                draw_text(
                    &mut self.buffer,
                    x,
                    0,
                    &text,
                    color,
                    false,
                );

                x += text.chars().count() as isize;
            }
        }

        // -------------------------
        // Current settings
        // -------------------------
        if self.show_config {
            let text = self.controller.config_text();
            
            if !text.is_empty() {
                if let Some(text_width) = text
                    .iter()
                    .map(|line| line.chars().count())
                    .max()
                {
                    let width = text_width + 2;
                    let height = text.len() + 2;

                    let x = self.buffer.width().saturating_sub(width) / 2;
                    let y = self.buffer.height().saturating_sub(height) / 2;
                    
                    draw_text_block(
                        &mut self.buffer, 
                        x as isize,
                        y as isize,
                        text,
                        Color::WHITE,
                        true,
                        "[ Current settings ]",
                        "[c]",
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
    LastActivePane,
    InactiveCommand,
    ActiveCommand,
}

impl FocusState {
    pub fn border_color(self) -> Color {
        match self {
            Self::InactivePane => Color::INACTIVE,
            Self::ActivePane => Color::ACTIVE,
            Self::LastActivePane => Color::LAST_ACTIVE,
            Self::InactiveCommand => Color::INACTIVE,
            Self::ActiveCommand => Color::ACTIVE,
        }
    }

    pub fn label_color(self) -> Color {
        match self {
            Self::InactivePane | LastActivePane => Color::LAST_ACTIVE,
            Self::ActivePane => Color::ACTIVE,
            _ => Color::INACTIVE,
        }
    }
}