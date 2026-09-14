use std::{collections::HashMap, io::{self, Write}};
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers}, 
    terminal::{disable_raw_mode, enable_raw_mode},
};

use super::{
    Command, 
    Session, 
    SessionOutput, 
    HELP_2D,
    HELP_3D,
    HELP_WAVEFORM,
    HELP_COMMANDS,
};
use crate::{
    ui::{
        ContentController,
        CommandController,
        FocusState,
        InteractionMode,
        KeyResult,
        Layout,
        Pane,
        PlotContent,
        PlotController,
        PlotMode,
        Rect,
        ResizeAxis,
        PaneController,
    },
    rendering::{Buffer, Color, draw_text, draw_text_block},
    terminal::{self as term, TerminalPresenter},
};


#[derive(Debug, PartialEq, Eq)]
pub enum AppControl {
    Continue,
    Quit,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InputMode {
    Pane,
    Command,
}

#[derive(Debug, PartialEq, Eq)]
enum CommandKeyResult {
    Changed,
    Submit,
    Cancel,
    Ignored,
}

#[derive(Debug, PartialEq, Eq)]
enum CommandEffect {
    None,
    Redraw,
    Quit,
}

enum SplitDirection {
    Rows,
    Columns,
}

pub struct TermviewApp {
    session: Session,
    presenter: TerminalPresenter,
    command_pane: Pane<CommandController>,
    command_cursor: usize,
    panes: HashMap<usize, Pane<ContentController>>,
    active_pane: usize,
    next_pane_id: usize,
    layout: Layout,
    input_mode: InputMode,
    command: String,
    history: Vec<String>,
    history_index: Option<usize>,
    draft: String,
    message: Option<String>,
    show_help: bool,
}

impl TermviewApp {
    pub fn new() -> Self {
        let (width, height) = term::canvas_dims();
        let area = Rect::new(0, 0, width, height);

        Self {
            session: Session,
            presenter: TerminalPresenter::new(width, height),
            command_pane: Pane::new(
                CommandController { text: String::new() },
                Rect::new(0, 0, 0, 0),
                InteractionMode::Static,
                FocusState::InactiveCommand,
            ),
            command_cursor: 0,
            panes: HashMap::from([(
                0,
                Pane::new(
                    ContentController::Plot(PlotController::default()),
                    area, 
                    InteractionMode::Static, 
                    FocusState::ActivePane,
                ),
            )]),
            active_pane: 0,
            next_pane_id: 1,
            layout: Layout::Leaf { pane_id: 0 },
            input_mode: InputMode::Pane,
            command: String::new(),
            history: Vec::new(),
            history_index: None,
            draft: String::new(),
            message: None,
            show_help: false,
        }
    }

    /**
     * Public entry method to begin in Pane / Command view
     */
    pub fn run(
        &mut self,
        output: &mut impl Write,
    ) -> io::Result<AppControl> {
        self.show_view(output)
    }

    pub fn execute(
        &mut self,
        command: Command,
        output: &mut impl Write,
    ) -> Result<AppControl, Box<dyn std::error::Error>> {
        match self.apply_command(command)? {
            CommandEffect::Quit => Ok(AppControl::Quit),

            CommandEffect::Redraw => {
                Ok(self.show_view(output)?)
            }

            CommandEffect::None => {
                if let Some(message) = &self.message {
                    writeln!(output, "{message}")?;
                }

                Ok(AppControl::Continue)
            }
        }
    }

    /**
     * Handle / apply a Command submitted in Command mode
     * - Based on SessionOutput from executing Command,
     *   can be None, Message, Plot2d, or Plot3d
     */
    fn apply_command(
        &mut self, 
        command: Command
    ) -> Result<CommandEffect, Box<dyn std::error::Error>> {
        self.message = None;

        let pane = self.panes
            .get_mut(&self.active_pane)
            .ok_or_else(|| io::Error::other("Active pane does not exist"))?;

        // Quit termview
        if matches!(command, Command::Quit) {
            return Ok(CommandEffect::Quit);
        }

        // Help overlay
        if matches!(command, Command::Help) {
            self.show_help = !self.show_help;
            return Ok(CommandEffect::Redraw);
        }

        // Current config overlay
        if matches!(command, Command::Config) {
            pane.show_config = !pane.show_config;
            return Ok(CommandEffect::Redraw);
        }

        // -------------------------
        // Execute parsed Command with Session
        // -------------------------
        let projection_changed = matches!(&command, Command::SetProjection(_));
        let viewport_3d_changed = matches!(
            &command,
            Command::SetView {
                z_bounds: Some(_),
                ..
            } | Command::SetViewAxis { .. }
        );

        let result = self.session.execute(&mut pane.controller, command)?;

        match result {
            SessionOutput::None => Ok(CommandEffect::None),

            SessionOutput::Redraw => {
                if projection_changed || viewport_3d_changed {
                    if let ContentController::Plot(plot) = &mut pane.controller {
                        if let PlotContent::ThreeD(surface) = &mut plot.content {
                            if viewport_3d_changed {
                                surface.rebuild_mesh(&plot.view_3d);
                                plot.view_3d.reset_camera();
                            }

                            if pane.buffer.width() >= 3 && pane.buffer.height() >= 3 {
                                plot.view_3d.fit_camera(&pane.buffer, &surface.mesh.vertices);
                            }
                        }
                    }
                }

                Ok(CommandEffect::Redraw)
            }

            SessionOutput::Message(msg) => {
                self.message = Some(msg);
                Ok(CommandEffect::None)
            }

            SessionOutput::PaneUpdated { title } => {
                pane.set_title(title);
                pane.mode = InteractionMode::Interactive;

                if let ContentController::Plot(plot) = &mut pane.controller {
                    match &mut plot.content {
                        PlotContent::TwoD(curve) => {
                            plot.view_2d.reset_view();

                            // Prepare displayed frames with reset X bounds
                            curve.resample(&plot.view_2d);

                            // Fit across the cycle, static curves use existing points
                            curve.fit_animation_view(&mut plot.view_2d, 129);
                        }

                        PlotContent::ThreeD(surface) => {
                            // Rebuild mesh around default view then fit viewport
                            plot.view_3d.reset_view();
                            surface.rebuild_mesh(&plot.view_3d);
                            plot.view_3d.fit_view(&surface.mesh.vertices);

                            // Fit camera to newly fit viewport bounds
                            plot.view_3d.reset_camera();
                            plot.view_3d.fit_camera(&pane.buffer, &surface.mesh.vertices);
                        }

                        PlotContent::Empty => {}
                    }
                }

                Ok(CommandEffect::Redraw)
            }
        }
    }

    /**
     * On Enter, submit command and parse it into a Command
     * type to be executed by Session and applied by the app
     */
    fn submit_command(
        &mut self
    ) -> Result<CommandEffect, Box<dyn std::error::Error>> {
        let command = Command::parse(self.command.trim())?;
        self.apply_command(command)
    }

    /**
     * Render Panes into app frame Buffer
     */
    fn present(&mut self, output: &mut impl Write) -> io::Result<()> {
        let typing = self.input_mode == InputMode::Command;
        
        // Show command text input with _ cursor
        // - Give precedence to messages over typing
        self.command_pane.controller.text = if let Some(message) = &self.message {
            format!(" {}", message.clone())     // show message if there is one
        } else if typing {
            let (before, after) =
                self.command.split_at(self.command_cursor);

            format!(" :{before}▏{after}")
        } else {
            " Press : to enter a command".to_string()
        };
        
        // Set CommandPane focus
        if typing {
            self.command_pane.set_focus(FocusState::ActiveCommand);
        } else {
            self.command_pane.set_focus(FocusState::InactiveCommand);
        }

        // Determine 2D / 3D / Waveform active pane help text
        let help_text = match &self.get_active_pane()?.controller {
            ContentController::Plot(plot) => match plot.active_plot_mode {
                PlotMode::TwoD => HELP_2D,
                PlotMode::ThreeD => HELP_3D,
            },
            ContentController::Waveform(_) => HELP_WAVEFORM,
        };

        let pane_ids = self.ordered_pane_ids();
        let frame = self.presenter.begin_frame();

        // Render panes / set pane focus
        for (index, id) in pane_ids.into_iter().enumerate() {
            // Set pane display number 1-indexed by ordered ID
            let pane = self.panes.get_mut(&id)
                .expect("Pane ID came from the pane map");
            pane.number = Some(index + 1);

            let focus = if id != self.active_pane {
                FocusState::InactivePane
            } else if typing {
                FocusState::LastActivePane
            } else {
                FocusState::ActivePane
            };

            pane.set_focus(focus);
            pane.render_into(frame);
        }

        self.command_pane.render_into(frame);

        // Draw app-wide keyboard help after the panes
        if frame.height() > 0 {
            let y = (frame.height() - 4) as isize;

            let x: isize = if help_text.chars().count() >= frame.width() {
                2 
            } else {
                ((frame.width() - help_text.chars().count()) as f32 / 2.0).round() as isize
            };

            draw_text(
                frame, 
                x, 
                y, 
                help_text, 
                Color::WHITE, 
                false
            );
        }

        // Show command help overlay in center of frame
        if self.show_help {
            let lines: Vec<String> = HELP_COMMANDS
                .trim_matches('\n')
                .lines()
                .map(|line| line.trim_end().to_string())
                .collect();

            let text_width = lines
                .iter()
                .map(|line| line.chars().count())
                .max()
                .unwrap_or(0);

            let width = text_width + 2;
            let height = lines.len() + 2;

            let mut overlay = Buffer::new(width, height);

            draw_text_block(
                &mut overlay,
                0,
                0,
                lines,
                Color::WHITE,
                true,
                "[ Help ]",
                "[h]",
            );

            let x = frame.width().saturating_sub(width) / 2;
            let y = frame.height().saturating_sub(height) / 2;

            frame.blit(&overlay, x, y);
        }

        self.presenter.present(output)
    }

    fn show_view(&mut self, output: &mut impl Write) -> io::Result<AppControl> {
        // Re-read dimensions / reset buffer if
        // the terminal was resized
        let (width, height) = term::canvas_dims();
        self.resize_view(width, height);

        // Clear / start drawing at top
        write!(
            output,
            "{}{}{}",
            term::CURSOR_HIDE,
            term::CLEAR_SCREEN,
            term::CURSOR_HOME,
        )?;

        let enable_raw_result = enable_raw_mode();

        // Display running view
        let view_result = self.run_view(output);
        let disable_raw_result = disable_raw_mode();

        // 1-indexed ANSI positions means
        // height + 1 is the reserved final row below plot
        let cursor_result = write!(
            output,
            "\x1b[{};1H{}",
            height + 1,
            term::CURSOR_SHOW,
        );

        let flush_result = output.flush();

        // run_view() AppControl output
        let control = view_result?;

        enable_raw_result?;
        disable_raw_result?;
        cursor_result?;
        flush_result?;

        Ok(control)
    }
    
    /**
     * 
     */
    fn run_view(
        &mut self, 
        output: &mut impl Write
    ) -> io::Result<AppControl> {
        self.present(output)?;

        // Init animation params
        let frame_interval = Duration::from_millis(16);
        let mut last_update = Instant::now();
        
        // Loop for keyboard inputs for interactive view modes
        loop {
            // Update animations when the next frame is due
            // after frame_interval time has elapsed
            let now = Instant::now();
            let elapsed = now.duration_since(last_update);

            if elapsed >= frame_interval {
                let delta_s = elapsed.as_secs_f64();
                last_update = now;

                // If any pane is updated, re-present the output
                // - controller.update() returns true when needing a redraw
                let mut changed = false;

                for pane in self.panes.values_mut() {
                    if pane.controller.update(delta_s) {
                        changed = true;
                    }
                }

                if changed {
                    self.present(output)?;
                }
            }

            // Wait until the next frame for processing input
            // - event::poll(timeout) waits up to `timeout` duration for an event
            let timeout = frame_interval.saturating_sub(last_update.elapsed());

            if !event::poll(timeout)? {
                continue;
            }

            // Handle key inputs
            match event::read()? {
                // Key-input terminal resize
                Event::Resize(width, height) => {
                    self.resize_view(
                        width as usize, 
                        height.saturating_sub(1) as usize
                    );

                    // Clear terminal
                    write!(output, "{}{}", term::CLEAR_SCREEN, term::CURSOR_HOME)?;

                    self.present(output)?;
                }

                Event::Key(key) => {
                    if key.kind == KeyEventKind::Release {
                        continue;
                    }

                    // -------------------------
                    // Ctrl + C to Quit
                    // -------------------------
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && matches!(key.code, KeyCode::Char('c' | 'C'))
                    {
                        return Ok(AppControl::Quit);
                    }

                    // -------------------------
                    // Command mode
                    // -------------------------
                    if self.input_mode == InputMode::Command {
                        match self.handle_command_key(key) {
                            CommandKeyResult::Changed => {
                                self.message = None;
                                self.present(output)?;
                            }

                            CommandKeyResult::Submit => {
                                // Empty command -> exit to Pane mode
                                if self.command.trim().is_empty() {
                                    self.input_mode = InputMode::Pane;
                                    self.command.clear();
                                } else {
                                    // Save to history before running so failed
                                    // commands can be recalled, skip repeats
                                    let entry = self.command.trim().to_string();

                                    if self.history.last() != Some(&entry) {
                                        self.history.push(entry);
                                    }

                                    match self.submit_command() {
                                        Ok(CommandEffect::Quit) => {
                                            return Ok(AppControl::Quit);
                                        }

                                        Ok(CommandEffect::Redraw) => {
                                            self.input_mode = InputMode::Pane;
                                            self.command.clear();
                                            self.command_cursor = 0;
                                        }

                                        Ok(CommandEffect::None) => {
                                            self.command.clear();
                                            self.command_cursor = 0;
                                        }

                                        Err(error) => {
                                            self.message = Some(error.to_string());
                                        }
                                    }
                                }

                                self.present(output)?;
                            }

                            CommandKeyResult::Cancel => {
                                self.input_mode = InputMode::Pane;
                                self.command.clear();
                                self.command_cursor = 0;
                                self.message = None;
                                self.present(output)?;
                            }

                            CommandKeyResult::Ignored => {}
                        }

                        continue;
                    }

                    // -------------------------
                    // Pane mode
                    // -------------------------
                    match key.code {
                        // Enter command mode
                        KeyCode::Char(':') | KeyCode::Char('x') => {
                            self.input_mode = InputMode::Command;
                            self.command.clear();
                            self.command_cursor = 0;
                            self.message = None;
        
                            self.present(output)?;
                            continue;
                        }

                        // Show help overlay
                        KeyCode::Char('h') | KeyCode::Char('?') => {
                            self.show_help = !self.show_help;
                            self.present(output)?;
                            continue; 
                        }

                        // Go to pane by number [1..9]
                        KeyCode::Char(c @ '1'..='9') => {
                            let number = (c as u8 - b'0') as usize;

                            if let Some(id) = self.pane_id_for_number(number) {
                                self.active_pane = id;
                                self.present(output)?;
                            }

                            continue;
                        }

                        // Next active pane
                        KeyCode::Tab => {
                            if self.panes.len() > 1 {
                                self.next_pane()?;
                            // One pane: switch to command pane
                            } else {
                                self.input_mode = InputMode::Command;
                                self.command.clear();
                                self.command_cursor = 0;
                                self.message = None;
                                self.get_active_pane()?.set_focus(FocusState::LastActivePane);
                            }
        
                            self.present(output)?;
                            continue
                        }

                        // Column split
                        KeyCode::Char('%') => {
                            self.split_active_pane(SplitDirection::Columns)?;

                            self.present(output)?;
                            continue;
                        }

                        // Row split
                        KeyCode::Char('"') => {
                            self.split_active_pane(SplitDirection::Rows)?;

                            self.present(output)?;
                            continue;
                        }

                        // -------------------------
                        // Resize pane
                        // -------------------------
                        KeyCode::Char(c @ ('j' | 'l' | 'i' | 'k')) => {
                            let (axis, amount) = match c {
                                'j' => (ResizeAxis::Horizontal, -0.05),
                                'l' => (ResizeAxis::Horizontal,  0.05),
                                'i' => (ResizeAxis::Vertical,   -0.05),
                                'k' => (ResizeAxis::Vertical,    0.05),
                                _ => unreachable!(),
                            };

                            if self.resize_active_pane(axis, amount) {
                                self.present(output)?;
                            }
                            continue;
                        }

                        // Delete pane
                        KeyCode::Delete | KeyCode::Backspace => {
                            self.close_active_pane()?;

                            self.present(output)?;
                            continue;
                        }

                        _ => {}
                    }

                    // -------------------------
                    // Key handled by the pane's controller
                    // -------------------------
                    let pane = self.get_active_pane()?;
                    let result = pane.handle_key(key);

                    // Reset camera by fitting to viewport
                    if key.code == KeyCode::Char('r')
                        && result == KeyResult::Changed
                        && pane.buffer.width() >= 3
                        && pane.buffer.height() >= 3
                    {
                        if let ContentController::Plot(plot) = &mut pane.controller {
                            if let PlotContent::ThreeD(surface) = &plot.content {
                                plot.view_3d.fit_camera(&pane.buffer, &surface.mesh.vertices);
                            }
                        }
                    }
                    
                    match result {
                        KeyResult::Changed => {
                            self.present(output)?
                        }
                        KeyResult::Exit => {
                            self.input_mode = InputMode::Command;
                            self.command.clear();
                            self.command_cursor = 0;
                            self.message = None;
                            
                            self.present(output)?;
                            continue;
                        }
                        KeyResult::Ignored => {},
                    }
                }

                _ => {}
            }
        }
    }

    /**
     * Handles command typing (e.g., writing plots in command Pane)
     * - Normal typing functionality, escape to exit
     */
    fn handle_command_key(&mut self, key: KeyEvent) -> CommandKeyResult {
        match key.code {
            // Normal equation typing
            KeyCode::Char(c) 
                if (c.is_ascii_alphanumeric()
                    || matches!(c, '+' | '-' | '*' | '/' | '^' | '(' | ')' | ' ' | '.' | '=')) => 
            {
                // Typing over a message starts a fresh command
                if self.message.take().is_some() {
                    self.command.clear();
                    self.command_cursor = 0;
                }

                self.command.insert(self.command_cursor, c);
                self.command_cursor += 1;
                CommandKeyResult::Changed
            }

            // Switch to Pane
            KeyCode::Tab => {
                self.input_mode = InputMode::Pane;
                CommandKeyResult::Changed
            }

            // Up / Down: Step through command history
            KeyCode::Up => self.recall_history(true),
            KeyCode::Down => self.recall_history(false),

            // Left / Right: Move cursor
            KeyCode::Left => {
                if self.command_cursor > 0 {
                    self.command_cursor -= 1;
                    CommandKeyResult::Changed
                } else {
                    CommandKeyResult::Ignored
                }
            }

            KeyCode::Right => {
                if self.command_cursor < self.command.len() {
                    self.command_cursor += 1;
                    CommandKeyResult::Changed
                } else {
                    CommandKeyResult::Ignored
                }
            }

            // Backspace: delete preceding character
            KeyCode::Backspace => {
                if self.command_cursor > 0 {
                    self.command_cursor -= 1;
                    self.command.remove(self.command_cursor);
                    CommandKeyResult::Changed
                } else {
                    CommandKeyResult::Ignored
                }
            }

            // Delete: delete current character
            KeyCode::Delete => {
                if self.command_cursor < self.command.len() {
                    self.command.remove(self.command_cursor);
                    CommandKeyResult::Changed
                } else {
                    CommandKeyResult::Ignored
                }
            }

            // Enter: Submit command
            KeyCode::Enter => {
                self.command_cursor = 0;
                CommandKeyResult::Submit
            }

            // Exit: Return to Pane mode
            KeyCode::Esc => {
                self.command_cursor = 0;
                CommandKeyResult::Cancel
            }

            _ => CommandKeyResult::Ignored,
        }
    }

    /**
     * Step to an older / newer submitted command
     * - Browsing only continues while the text still matches the
     *   recalled entry, so any edit or clear starts a new draft
     * - Stepping past the newest entry restores the draft
     */
    fn recall_history(&mut self, older: bool) -> CommandKeyResult {
        let browsing = self.history_index
            .filter(|&i| self.history.get(i) == Some(&self.command));

        let index = match (browsing, older) {
            (None, true) if !self.history.is_empty() => {
                self.draft = self.command.clone();
                Some(self.history.len() - 1)
            }
            (None, _) => return CommandKeyResult::Ignored,

            (Some(i), true) => Some(i.saturating_sub(1)),
            (Some(i), false) if i + 1 < self.history.len() => Some(i + 1),
            (Some(_), false) => None,
        };

        self.history_index = index;
        self.command = match index {
            Some(i) => self.history[i].clone(),
            None => std::mem::take(&mut self.draft),
        };
        self.command_cursor = self.command.len();

        CommandKeyResult::Changed
    }

    /**
     * Update the Pane layouts recursively
     */
    fn relayout(&mut self, width: usize, height: usize) {
        // Reserve 3 lines for the command pane (borders + text row)
        let command_height = height.min(3);
        let plot_height = height - command_height;

        // Start left (x=0) and below plot pane (y=plot_height)
        self.command_pane.set_area(Rect::new(
            0,
            plot_height,
            width,
            command_height,
        ));

        let plot_area = Rect::new(0, 0, width, plot_height);
        let panes = &mut self.panes;

        self.layout.visit_areas(plot_area, &mut |id, area| {
            if let Some(pane) = panes.get_mut(&id) {
                pane.set_area(area);
            }
        });
    }
    
    /**
     * Resize view of full terminal screen, adjusting
     * sub-panes recursively
     */
    fn resize_view(&mut self, width: usize, height: usize) {
        self.presenter.resize(width, height);
        self.relayout(width, height);
    }

    /**
     * Resize active pane by adjusting its first sibling's fraction,
     * relayout adjusts the children accordingly
     */
    fn resize_active_pane(
        &mut self, 
        axis: ResizeAxis,
        amount: f32,
    ) -> bool {
        if !self.layout.resize_pane(self.active_pane, axis, amount) {
            return false;
        }

        let (width, height) = term::canvas_dims();
        self.relayout(width, height);
        true
    }

    /**
     * Splits the active pane vertically (Columns) or horizontally (Rows),
     * updates the active pane with the replacement split Layout in the tree 
     * and updates its size to the new split size
     */
    fn split_active_pane(
        &mut self,
        direction: SplitDirection,
    ) -> io::Result<()> {
        let active_id = self.active_pane;
        let next_id = self.next_pane_id;

        let active_area = self.get_active_pane()?.area;
        let fraction = 0.5;

        // Get new area for active pane, new pane area, and replacement Layout
        let (new_active_area, next_area, split_layout) = match direction {
            SplitDirection::Rows => {
                let (top, bottom) = active_area.split_rows(fraction);

                (top, bottom, Layout::RowSplit {
                    fraction,
                    top: Box::new(Layout::Leaf { pane_id: active_id }),
                    bottom: Box::new(Layout::Leaf { pane_id: next_id }),
                })
            }

            SplitDirection::Columns => {
                let (left, right) = active_area.split_cols(fraction);

                (left, right, Layout::ColumnSplit {
                    fraction,
                    left: Box::new(Layout::Leaf { pane_id: active_id }),
                    right: Box::new(Layout::Leaf { pane_id: next_id }),
                })
            }
        };

        // Update active pane Layout leaf
        let leaf = self.layout.find_leaf_mut(active_id)
            .ok_or_else(|| io::Error::other("Active pane is missing from layout"))?;

        *leaf = split_layout;

        // Update active pane's area to its split area & set inactive
        let active_pane = self.panes.get_mut(&active_id).unwrap();

        active_pane.set_area(new_active_area);
        active_pane.set_focus(FocusState::InactivePane);

        // Insert newly split next Pane into hashmap
        self.panes.insert(
            next_id,
            Pane::new(
                ContentController::Plot(PlotController::default()),
                next_area,
                InteractionMode::Static,
                FocusState::ActivePane,
            ),
        );

        // Update next pane id / set active pane to newly split id
        self.next_pane_id += 1;
        self.active_pane = next_id;

        Ok(())
    }

    /**
     * Set active pane to next pane ID in Panes HashMap indices
     */
    fn next_pane(&mut self) -> io::Result<()> {
        let pane_ids = self.ordered_pane_ids();

        // Get active pand ID's HashMap index
        let active_idx = pane_ids.iter()
            .position(|&id| id == self.active_pane)
            .ok_or_else(|| io::Error::other("Active pane does not exist"))?;

        self.get_active_pane()?.set_focus(FocusState::InactivePane);    // Current -> Inactive
        self.active_pane = pane_ids[(active_idx + 1) % pane_ids.len()];
        self.get_active_pane()?.set_focus(FocusState::ActivePane);      // Next -> Active

        Ok(())
    }

    /**
     * Close the active pane and replace its parent split with its sibling
     * within the Layout tree. Set's the surviving subtree's first pane id
     * as the new active pane id
     */
    fn close_active_pane(&mut self) -> io::Result<()> {
        // Always keep one pane open
        if self.panes.len() <= 1 {
            return Ok(());
        }

        // Get surviving subtree's first pane id
        let closing_id = self.active_pane;
        let survivor_id = self.layout.remove_leaf(closing_id)
            .ok_or_else(|| io::Error::other("Active pane could not be removed from layout"))?;

        // Remove pane from HashMap & Session plot history
        self.panes.remove(&closing_id);

        self.active_pane = survivor_id;
        self.get_active_pane()?.set_focus(FocusState::ActivePane);

        // Resize panes
        let (width, height) = term::canvas_dims();
        self.relayout(width, height);

        Ok(())
    }

    /**
     * Mutably get active pane, otherwise error if not found
     */
    fn get_active_pane(&mut self) -> io::Result<&mut Pane<ContentController>> {
        self.panes
            .get_mut(&self.active_pane)
            .ok_or_else(|| io::Error::other("Active pane does not exist").into())
    }

    /**
     * Get Pane IDs in sorted order
     */
    fn ordered_pane_ids(&self) -> Vec<usize> {
        let mut ids: Vec<_> = self.panes.keys().copied().collect();
        ids.sort_unstable();
        ids
    }

    /**
     * Given displayed Pane number (1, 2, 3, ...), get
     * the corresponding index in internal ordered Pane IDs
     */
    fn pane_id_for_number(&self, number: usize) -> Option<usize> {
        let index = number.checked_sub(1)?;
        self.ordered_pane_ids().get(index).copied()
    }
}

impl Default for TermviewApp {
    fn default() -> Self {
        Self::new()
    }
}
