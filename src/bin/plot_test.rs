use std::io::self;

use termview::{
    geometry::Point,
    rendering::{Cell, Color, PlotRenderer, PlotViewport},
    terminal::{self as term, TerminalPresenter},
    math::rangef,
};

fn main() -> io::Result<()> {
    let (width, height) = term::canvas_dims();

    let mut presenter = TerminalPresenter::new(width, height);

    let mut out = io::stdout().lock();

    let plot_color = Color::Rgb(80, 220, 120);
    let ticks_color = Color::Rgb(220, 220, 160);

    let renderer = PlotRenderer {
        line_cell: Cell::new('*').with_fg(plot_color),
        point_cell: Cell::new('@').with_fg(plot_color),

        ticks_color: ticks_color,

        pad_width: 0,
        pad_height: 5,
        ..Default::default()
    };

    let viewport = PlotViewport {
        x_min: -10.0,
        x_max: 10.0,
        y_min: -1.0,
        y_max: 1.0,
    };

    let points: Vec<Point> = rangef(-10.0, 10.0, 50).into_iter()
        .map(|n| Point {
            x: n,
            y: n.sin(),
        })
        .collect();

    term::enter_fullscreen(&mut out)?;

    let buffer = presenter.begin_frame();
    renderer.render(&points, &viewport, buffer);

    presenter.present(&mut out)?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    term::leave_fullscreen(&mut out)?;

    Ok(())
}