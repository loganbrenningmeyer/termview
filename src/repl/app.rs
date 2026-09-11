use std::{collections::HashMap, io::{self, Write}};
use std::time::{Duration, Instant};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind}, 
    terminal::{disable_raw_mode, enable_raw_mode},
};

use super::{Command, Session, SessionOutput};
use crate::{
    ui::{
        CommandWidget,
        FocusState,
        InteractionMode,
        KeyResult,
        Layout,
        Pane,
        PaneContent,
        Rect,
        Widget,
    },
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
    command_pane: Pane<CommandWidget>,
    panes: HashMap<usize, Pane<PaneContent>>,
    active_pane: usize,
    next_pane_id: usize,
    layout: Layout,
    input_mode: InputMode,
    command: String,
    message: Option<String>,
}

impl TermviewApp {
    pub fn new() -> Self {
        let (width, height) = term::canvas_dims();
        let area = Rect::new(0, 0, width, height);

        Self {
            session: Session::default(),
            presenter: TerminalPresenter::new(width, height),
            command_pane: Pane::new(
                Rect::new(0, 0, 0, 0),
                InteractionMode::Static,
                FocusState::Inactive,
                CommandWidget { text: String::new() },
            ),
            panes: HashMap::from([(
                0,
                Pane::new(
                    area, 
                    InteractionMode::Static, 
                    FocusState::Active,
                    PaneContent::Empty,
                ),
            )]),
            active_pane: 0,
            next_pane_id: 1,
            layout: Layout::Leaf { pane_id: 0 },
            input_mode: InputMode::Pane,
            command: String::new(),
            message: None,
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
    fn apply_command(&mut self, command: Command) -> Result<CommandEffect, Box<dyn std::error::Error>> {
        self.message = None;

        if matches!(command, Command::Quit) {
            return Ok(CommandEffect::Quit);
        }

        // -------------------------
        // Animation Pause / Resume / Speed & Time
        // -------------------------
        if matches!(
            &command,
            Command::Pause | Command::Resume |
            Command::SetSpeed(_) | Command::SetTime(_)
        ) {
            let pane = self.get_active_pane()?;
            let content = &mut pane.widget;

            let animation = content
                .animation_mut()
                .ok_or("The active pane has no animation")?;
        
            match &command {
                Command::Pause => animation.playing = false,
                Command::Resume => animation.playing = true,
                
                Command::SetSpeed(speed) => {
                    if !speed.is_finite() {
                        return Err("Animation speed must be finite".into());
                    }
                    
                    animation.speed = *speed;
                }
                
                Command::SetTime(time) => {
                    if !time.is_finite() {
                        return Err("Animation time must be finite".into())
                    }
                    animation.time = *time;
                    animation.phase = *time;
                    
                    // Time elapsed, resample animation
                    if matches!(&command, Command::SetTime(_)) {
                        content.resample();
                    }
                }

                _ => unreachable!(),
            }

            return Ok(CommandEffect::Redraw);   // redraw after animation updates
        }

        let result = self.session.execute(self.active_pane, command)?;

        // -------------------------
        // Execute parsed Command with Session
        // -------------------------
        match result {
            SessionOutput::None => Ok(CommandEffect::None),

            SessionOutput::Message(msg) => {
                self.message = Some(msg);
                Ok(CommandEffect::None)
            }

            SessionOutput::Plot { title, content } => {
                let pane = self.get_active_pane()?;

                pane.set_title(title);
                pane.set_widget(content);
                pane.mode = InteractionMode::Interactive;

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
        self.command_pane.widget.text = if let Some(message) = &self.message {
            format!(" {}", message.clone())     // show message if there is one
        } else if typing {
            format!(" :{}_", self.command)
        } else {
            " Press : to enter a command".to_string()
        };
        
        self.command_pane.set_focus(if typing {
            FocusState::Active
        } else {
            FocusState::Inactive
        });

        let frame = self.presenter.begin_frame();

        for pane in self.panes.values_mut() {
            pane.render_into(frame);
        }

        self.command_pane.render_into(frame);

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
                // - widget.update() returns true when needing a redraw
                let mut changed = false;

                for pane in self.panes.values_mut() {
                    if pane.widget.update(delta_s) {
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
                    self.resize_view(width as usize, height as usize);
                    self.present(output)?;
                }

                Event::Key(key) => {
                    if key.kind == KeyEventKind::Release {
                        continue;
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
                                    match self.submit_command() {
                                        Ok(CommandEffect::Quit) => {
                                            return Ok(AppControl::Quit);
                                        }

                                        Ok(CommandEffect::Redraw) => {
                                            self.input_mode = InputMode::Pane;
                                            self.command.clear();
                                        }

                                        Ok(CommandEffect::None) => {
                                            self.command.clear();
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
                                self.message = None;
                                self.present(output)?;
                            }

                            CommandKeyResult::Ignored => {}
                        }

                        continue;
                    }

                    // -------------------------
                    // Enter Command mode from Pane mode
                    // -------------------------
                    if key.code == KeyCode::Char(':') {
                        self.input_mode = InputMode::Command;
                        self.command.clear();
                        self.message = None;

                        self.present(output)?;
                        continue;
                    }

                    // -------------------------
                    // Next active pane
                    // -------------------------
                    if key.code == KeyCode::Tab {
                        self.next_pane()?;

                        self.present(output)?;
                        continue
                    }

                    // -------------------------
                    // Split pane / Delete pane
                    // -------------------------
                    match key.code {
                        // Column split
                        KeyCode::Char('\\') => {
                            self.split_active_pane(SplitDirection::Columns)?;

                            self.present(output)?;
                            continue;
                        }

                        // Row split
                        KeyCode::Char('-') => {
                            self.split_active_pane(SplitDirection::Rows)?;

                            self.present(output)?;
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
                    // Key handled by Pane Widget
                    // -------------------------
                    let pane = self.get_active_pane()?;
                    let result = pane.handle_key(key);
                    
                    match result {
                        KeyResult::Changed => {
                            self.present(output)?
                        }
                        KeyResult::Exit => {
                            self.input_mode = InputMode::Pane;
                            self.command.clear();
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
                if (c.is_alphanumeric() 
                    || matches!(c, '+' | '-' | '*' | '/' | '^' | '(' | ')' | ' ' | '.' | '=')) => 
            {
                self.command.push(c);
                CommandKeyResult::Changed
            }

            // Backspace: Delete character
            KeyCode::Backspace | KeyCode::Delete => {
                if self.command.pop().is_some() {
                    CommandKeyResult::Changed
                } else {
                    CommandKeyResult::Ignored
                }
            }

            // Enter: Submit command
            KeyCode::Enter => CommandKeyResult::Submit,

            // Exit: Return to Pane mode
            KeyCode::Esc => CommandKeyResult::Cancel,

            _ => CommandKeyResult::Ignored,
        }
    }

    /**
     * Update the Pane layouts recursively
     */
    fn relayout(&mut self, width: usize, height: usize) {
        // Reserve 3 lines for command widget (borders + text row)
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
        active_pane.set_focus(FocusState::Inactive);

        // Insert newly split next Pane into hashmap
        self.panes.insert(
            next_id,
            Pane::new(
                next_area,
                InteractionMode::Static,
                FocusState::Active,
                PaneContent::Empty,
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
        let mut pane_indices: Vec<usize> = self.panes.keys().copied().collect();
        pane_indices.sort_unstable();

        // Get active pand ID's HashMap index
        let active_idx = pane_indices.iter()
            .position(|&id| id == self.active_pane)
            .ok_or_else(|| io::Error::other("Active pane does not exist"))?;

        self.get_active_pane()?.set_focus(FocusState::Inactive);    // Current -> Inactive
        self.active_pane = pane_indices[(active_idx + 1) % pane_indices.len()];
        self.get_active_pane()?.set_focus(FocusState::Active);      // Next -> Active

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
        self.session.forget_pane(closing_id);

        self.active_pane = survivor_id;
        self.get_active_pane()?.set_focus(FocusState::Active);

        // Resize panes
        let (width, height) = term::canvas_dims();
        self.relayout(width, height);

        Ok(())
    }

    /**
     * Mutably get active pane, otherwise error if not found
     */
    fn get_active_pane(&mut self) -> io::Result<&mut Pane<PaneContent>> {
        self.panes
            .get_mut(&self.active_pane)
            .ok_or_else(|| io::Error::other("Active pane does not exist").into())
    }
}

impl Default for TermviewApp {
    fn default() -> Self {
        Self::new()
    }
}
