use std::{
    f64::consts::TAU,
    io,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use termview::{
    geometry::Point,
    math::rangef,
    rendering::{Color, PlotAspect2d, PlotRenderer2d, PlotStyle2d, PlotViewport2d},
    terminal::{self as term, TerminalPresenter},
};

const CELL_ASPECT: f64 = 0.5;
const FRAME_TIME: Duration = Duration::from_micros(16_667);
const DEMO_DURATION: Duration = Duration::from_secs(30);

fn main() -> io::Result<()> {
    let viewport = PlotViewport2d {
        x_min: -4.0,
        x_max: 4.0,
        y_min: -4.0,
        y_max: 4.0,
    };

    let (width, height) = term::canvas_viewport_dims(&viewport, CELL_ASPECT);
    let mut presenter = TerminalPresenter::new(width, height);
    let mut out = io::stdout().lock();

    let tick_color = Color::Rgb(220, 220, 160);
    let mut renderer = PlotRenderer2d {
        style: PlotStyle2d::default()
            .curve_color(Color::Rgb(80, 220, 120))
            .tick_color(tick_color),
        aspect: PlotAspect2d::Equal {
            cell_aspect: CELL_ASPECT,
        },
        pad_width: 5,
        pad_height: 5,
        ..Default::default()
    };

    // A heart gives the rotation an obvious orientation and stays within
    // the square viewport throughout the animation.
    let base_points: Vec<Point> = rangef(0.0, TAU, 320)
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

    // Enter exits early too, while the duration is a fallback that ensures
    // the terminal is restored even if no input is provided.
    let (exit_tx, exit_rx) = mpsc::channel();
    thread::spawn(move || {
        let mut input = String::new();
        let _ = io::stdin().read_line(&mut input);
        let _ = exit_tx.send(());
    });

    term::enter_fullscreen(&mut out)?;

    let animation_result = (|| -> io::Result<()> {
        let started = Instant::now();
        let mut points = Vec::with_capacity(base_points.len());

        loop {
            let frame_started = Instant::now();
            let elapsed = started.elapsed();

            if elapsed >= DEMO_DURATION || exit_rx.try_recv().is_ok() {
                break;
            }

            let seconds = elapsed.as_secs_f64();
            let angle = seconds * 0.8;
            let (sin_angle, cos_angle) = angle.sin_cos();
            let pulse = 1.0 + 0.06 * (seconds * 2.0).sin();

            points.clear();
            points.extend(base_points.iter().map(|point| Point {
                x: pulse * (point.x * cos_angle - point.y * sin_angle),
                y: pulse * (point.x * sin_angle + point.y * cos_angle),
            }));

            // Cycling the curve color also verifies that color-only cell
            // changes are detected by the front/back buffer comparison.
            let curve_color = Color::Rgb(
                (150.0 + 100.0 * (seconds * 0.7).sin()) as u8,
                (150.0 + 100.0 * (seconds * 0.7 + 2.1).sin()) as u8,
                (150.0 + 100.0 * (seconds * 0.7 + 4.2).sin()) as u8,
            );
            renderer.style = renderer.style.curve_color(curve_color);

            let buffer = presenter.begin_frame();
            renderer.render(&points, &viewport, buffer);
            presenter.present(&mut out)?;

            thread::sleep(FRAME_TIME.saturating_sub(frame_started.elapsed()));
        }

        Ok(())
    })();

    let leave_result = term::leave_fullscreen(&mut out);
    animation_result.and(leave_result)
}
