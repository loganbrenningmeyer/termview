//! Run with `cargo run --bin pane_demo` in a terminal.
//! Tab/1/2/3 selects a pane; WASD/arrows pan or orbit; E/Q zooms.
//! [/] adjusts the top share; ,/. adjusts the bottom-left share; X/Esc exits.

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, size},
};
use std::{
    collections::HashMap,
    io::{self, Write},
};
use termview::{
    geometry::{Mesh, Object, Point},
    rendering::{
        Buffer, Camera, Cell, Color, PlotRenderer2d, PlotRenderer3d, PlotViewport2d, PlotViewport3d,
    },
    terminal::{self as term, TerminalPresenter},
    ui::{FocusState, InteractionMode, KeyResult, Layout, Pane, PaneContent, PlotWidget2d, PlotWidget3d, Rect},
};

const TITLES: [&str; 3] = ["1: 3D surface", "2: sin(x)", "3: cos(x)"];

struct Demo {
    panes: HashMap<usize, Pane<PaneContent>>,
    presenter: TerminalPresenter,
    active: usize,
    width: usize,
    height: usize,
    top_fraction: f32,
    left_fraction: f32,
}

impl Demo {
    fn new(width: usize, height: usize) -> Self {
        let surface = Object::new(
            Mesh::surface(-3.0, 3.0, -3.0, 3.0, 35, 35, |x, y| x.sin() * y.cos()),
            Default::default(),
        );
        let plot = PlotWidget3d::new(
            surface,
            Camera::default(),
            PlotViewport3d {
                x_min: -3.0,
                x_max: 3.0,
                y_min: -3.0,
                y_max: 3.0,
                z_min: -1.0,
                z_max: 1.0,
            },
            PlotRenderer3d {
                show_border: true,
                ..Default::default()
            },
        );
        let empty_area = Rect::new(0, 0, 0, 0);
        let contents = [PaneContent::Plot3d(plot), curve(false), curve(true)];
        let panes = contents
            .into_iter()
            .enumerate()
            .map(|(id, content)| {
                (
                    id,
                    Pane::new(empty_area, InteractionMode::Interactive, FocusState::Active, content),
                )
            })
            .collect();
        let mut demo = Self {
            panes,
            presenter: TerminalPresenter::new(width, height),
            active: 0,
            width,
            height,
            top_fraction: 0.6,
            left_fraction: 0.4,
        };
        demo.layout();
        demo
    }

    fn layout(&mut self) {
        let layout = Layout::RowSplit {
            fraction: self.top_fraction,
            top: Box::new(Layout::Leaf { pane_id: 0 }),
            bottom: Box::new(Layout::ColumnSplit {
                fraction: self.left_fraction,
                left: Box::new(Layout::Leaf { pane_id: 1 }),
                right: Box::new(Layout::Leaf { pane_id: 2 }),
            }),
        };
        // Two rows belong to demo status/help, outside the pane layout.
        let area = Rect::new(0, 0, self.width, self.height.saturating_sub(2));
        layout.visit_areas(area, &mut |id, area| {
            self.panes
                .get_mut(&id)
                .expect("layout pane exists")
                .set_area(area);
        });
    }

    fn present(&mut self, output: &mut impl Write) -> io::Result<()> {
        let frame = self.presenter.begin_frame();
        for (id, pane) in &mut self.panes {
            pane.render_into(frame);
            if pane.area.height > 0 {
                let color = if *id == self.active {
                    Color::Rgb(255, 220, 80)
                } else {
                    Color::Rgb(160, 160, 160)
                };
                let title = format!(
                    " {}{} ",
                    if *id == self.active { "> " } else { "" },
                    TITLES[*id]
                );
                text(
                    frame,
                    pane.area.x,
                    pane.area.y,
                    pane.area.width,
                    &title,
                    color,
                );
            }
        }
        if self.height >= 2 {
            let status = format!(
                "Focus: {} | top {:.0}% | bottom-left {:.0}% | Tab/1/2/3: focus",
                TITLES[self.active],
                self.top_fraction * 100.0,
                self.left_fraction * 100.0
            );
            text(
                frame,
                0,
                self.height - 2,
                self.width,
                &status,
                Color::Rgb(255, 220, 80),
            );
            text(
                frame,
                0,
                self.height - 1,
                self.width,
                "WASD/arrows: move | E/Q: zoom | [/]: top | ,/.: left | X/Esc: exit",
                Color::Default,
            );
        }
        self.presenter.present(output)?;
        // Even an empty frame must flush a preceding resize clear.
        output.flush()
    }

    fn run(&mut self, output: &mut impl Write) -> io::Result<()> {
        self.present(output)?;
        loop {
            match event::read()? {
                Event::Resize(width, height) => {
                    self.width = usize::from(width);
                    self.height = usize::from(height);
                    self.presenter.resize(self.width, self.height);
                    self.layout();
                    // Resizing resets the presenter's knowledge of old cells.
                    write!(output, "{}{}", term::CLEAR_SCREEN, term::CURSOR_HOME)?;
                }
                Event::Key(key) if key.kind != KeyEventKind::Release => match key.code {
                    KeyCode::Esc | KeyCode::Char('x') => break,
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => break,
                    KeyCode::Tab => self.active = (self.active + 1) % 3,
                    KeyCode::BackTab => self.active = (self.active + 2) % 3,
                    KeyCode::Char(ch @ '1'..='3') => self.active = (ch as u8 - b'1') as usize,
                    KeyCode::Char(ch @ ('[' | ']' | ',' | '.')) => {
                        let (fraction, delta) = match ch {
                            '[' => (&mut self.top_fraction, -0.05),
                            ']' => (&mut self.top_fraction, 0.05),
                            ',' => (&mut self.left_fraction, -0.05),
                            _ => (&mut self.left_fraction, 0.05),
                        };
                        *fraction = (*fraction + delta).clamp(0.15, 0.85);
                        self.layout();
                    }
                    _ => {
                        if self
                            .panes
                            .get_mut(&self.active)
                            .expect("active pane exists")
                            .handle_key(key)
                            == KeyResult::Ignored
                        {
                            continue;
                        }
                    }
                },
                _ => continue,
            }
            self.present(output)?;
        }
        Ok(())
    }
}

fn curve(cosine: bool) -> PaneContent {
    let points = (0..=400)
        .map(|i| {
            let x = -6.0 + 12.0 * f64::from(i) / 400.0;
            Point::new(x, if cosine { x.cos() } else { x.sin() })
        })
        .collect();
    PaneContent::Plot2d(PlotWidget2d::new(
        points,
        PlotViewport2d {
            x_min: -6.0,
            x_max: 6.0,
            y_min: -1.5,
            y_max: 1.5,
        },
        PlotRenderer2d {
            show_border: true,
            ..Default::default()
        },
    ))
}

// Clip labels to their pane so narrow layouts cannot overwrite neighbors.
fn text(frame: &mut Buffer, x: usize, y: usize, width: usize, value: &str, color: Color) {
    for (offset, ch) in value.chars().take(width).enumerate() {
        frame.set(
            (x + offset) as isize,
            y as isize,
            Cell::new(ch).with_fg(color),
        );
    }
}

fn main() -> io::Result<()> {
    let (width, height) = size()?;
    let mut demo = Demo::new(usize::from(width), usize::from(height));
    enable_raw_mode()?;
    let mut output = io::stdout().lock();
    // Save errors until all terminal cleanup has been attempted.
    let result = term::enter_fullscreen(&mut output).and_then(|()| demo.run(&mut output));
    let raw_result = disable_raw_mode();
    let screen_result = term::leave_fullscreen(&mut output);
    result?;
    raw_result?;
    screen_result
}
