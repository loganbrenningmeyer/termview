use std::io;

use termview::{
    geometry::Point,
    math::rangef,
    rendering::{Color, PlotAspect, PlotRenderer, PlotStyle, PlotViewport},
    terminal::{self as term, TerminalPresenter},
};

fn main() -> io::Result<()> {
    let viewport = PlotViewport {
        x_min: -4.0,
        x_max: 4.0,
        y_min: -4.0,
        y_max: 4.0,
    };

    let cell_aspect = 0.5;

    let (width, height) = term::canvas_viewport_dims(&viewport, cell_aspect);

    let mut presenter = TerminalPresenter::new(width, height);

    let mut out = io::stdout().lock();

    let plot_color = Color::Rgb(80, 220, 120);
    let ticks_color = Color::Rgb(220, 220, 160);

    let style = PlotStyle::default()
        .curve_color(plot_color)
        .tick_color(ticks_color);

    let renderer = PlotRenderer {
        style,
        aspect: PlotAspect::Equal { cell_aspect },
        pad_width: 5,
        pad_height: 5,
        ..Default::default()
    };

    use std::f64::consts::TAU;

    let points: Vec<Point> = rangef(0.0, TAU, 400)
        .into_iter()
        .map(|t| Point {
            x: 0.2 * 16.0 * t.sin().powi(3),
            y: 0.2
                * (13.0 * t.cos()
                    - 5.0 * (2.0 * t).cos()
                    - 2.0 * (3.0 * t).cos()
                    - (4.0 * t).cos()),
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
