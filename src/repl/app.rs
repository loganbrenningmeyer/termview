use std::io::{self, Write};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode},
};

use crate::geometry::{Object, Point};
use crate::terminal::{self as term, TerminalPresenter};
use super::{Command, Session, SessionOutput};


#[derive(Debug, PartialEq, Eq)]
pub enum AppControl {
    Continue,
    Quit,
}

pub struct TermviewApp {
    session: Session,
    presenter: TerminalPresenter,
}

impl TermviewApp {
    pub fn new() -> Self {
        let (width, height) = term::canvas_dims();

        Self {
            session: Session::default(),
            presenter: TerminalPresenter::new(width, height),
        }
    }

    pub fn execute(
        &mut self,
        command: Command,
        output: &mut impl Write,
    ) -> Result<AppControl, Box<dyn std::error::Error>> {
        if matches!(command, Command::Quit) {
            return Ok(AppControl::Quit);
        }

        match self.session.execute(command)? {
            SessionOutput::None => {}

            SessionOutput::Message(msg) => {
                writeln!(output, "{msg}")?;
            }

            SessionOutput::Plot2d(points) => {
                self.present_plot(&points, output)?;
            }

            SessionOutput::Plot3d(surface) => {
                self.present_plot3d(&surface, output)?;
            }
        }

        Ok(AppControl::Continue)
    }

    /**
     * Plot 2D points to output stream
     */
    fn present_plot(
        &mut self,
        points: &Vec<Point>,
        output: &mut impl Write,
    ) -> io::Result<()> {
        // Re-read dimensions / reset buffer if
        // the terminal was resized
        let (width, height) = term::canvas_dims();
        self.presenter.resize(width, height);

        // Clear / start drawing at top
        write!(
            output,
            "{}{}{}",
            term::CURSOR_HIDE,
            term::CLEAR_SCREEN,
            term::CURSOR_HOME,
        )?;

        let buffer = self.presenter.begin_frame();
        self.session.render_2d(points, buffer);
        self.presenter.present(output)?;

        // 1-indexed ANSI positions means
        // height + 1 is the reserved final row below plot
        write!(
            output,
            "\x1b[{};1H{}",
            height + 1,
            term::CURSOR_SHOW,
        )?;

        output.flush()
    }

    /**
     * Plot surface of 3D points to output stream
     */
    fn present_plot3d(
        &mut self,
        surface: &Object,
        output: &mut impl Write,
    ) -> io::Result<()> {
        let (width, height) = term::canvas_dims();
        self.presenter.resize(width, height);

        write!(
            output,
            "{}{}{}",
            term::CURSOR_HIDE,
            term::CLEAR_SCREEN,
            term::CURSOR_HOME,
        )?;

        enable_raw_mode()?;

        let view_result = self.run_plot3d_view(surface, output);
        let raw_mode_result = disable_raw_mode();

        // Restore the prompt row even if rendering or input failed.
        let cursor_result = write!(
            output,
            "\x1b[{};1H{}",
            height + 1,
            term::CURSOR_SHOW,
        );
        let flush_result = output.flush();

        view_result?;
        raw_mode_result?;
        cursor_result?;
        flush_result
    }

    fn run_plot3d_view(
        &mut self,
        surface: &Object,
        output: &mut impl Write,
    ) -> io::Result<()> {
        self.draw_plot3d(surface, output)?;

        loop {
            let Event::Key(key) = event::read()? else {
                continue;
            };

            if key.kind == KeyEventKind::Release {
                continue;
            }

            let changed = match key.code {
                // -------------------------
                // Camera Orbit
                // -------------------------
                KeyCode::Left | KeyCode::Char('a') => {
                    self.session.orbit_camera_3d(-5.0, 0.0);
                    true
                }
                KeyCode::Right | KeyCode::Char('d') => {
                    self.session.orbit_camera_3d(5.0, 0.0);
                    true
                }
                KeyCode::Up | KeyCode::Char('w') => {
                    self.session.orbit_camera_3d(0.0, 5.0);
                    true
                }
                KeyCode::Down | KeyCode::Char('s') => {
                    self.session.orbit_camera_3d(0.0, -5.0);
                    true
                }

                // -------------------------
                // Camera Zoom
                // -------------------------
                // Zoom in
                KeyCode::Char('e') => {
                    self.session.zoom_camera_3d(-1.0);
                    true
                }
                // Zoom out
                KeyCode::Char('q') => {
                    self.session.zoom_camera_3d(1.0);
                    true
                }

                // -------------------------
                // Reset camera to default
                // -------------------------
                KeyCode::Char('r') => {
                    self.session.reset_camera_3d();
                    true
                }

                // -------------------------
                // Exit plot
                // -------------------------
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('x') => break,
                _ => false,
            };

            if changed {
                self.draw_plot3d(surface, output)?;
            }
        }

        Ok(())
    }

    fn draw_plot3d(
        &mut self,
        surface: &Object,
        output: &mut impl Write,
    ) -> io::Result<()> {
        let buffer = self.presenter.begin_frame();
        self.session.render_3d(surface, buffer);
        self.presenter.present(output)
    }
}

impl Default for TermviewApp {
    fn default() -> Self {
        Self::new()
    }
}
