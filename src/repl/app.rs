use std::io::{self, Write};

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

        let buffer = self.presenter.begin_frame();
        self.session.render_3d(surface, buffer);
        self.presenter.present(output)?;

        write!(
            output,
            "\x1b[{};1H{}",
            height + 1,
            term::CURSOR_SHOW,
        )?;

        output.flush()
    }
}

impl Default for TermviewApp {
    fn default() -> Self {
        Self::new()
    }
}
