use std::time::Instant;
use std::thread;
use std::io::self;
use std::time::Duration;

use termview::{
    geometry::{Mesh, Object},
    math::{Quaternion, Transform, Vec3},
    rendering::{Camera, WireframeRenderer, WireframeStyle},
    terminal::{self as term, TerminalPresenter},
};


pub struct AppConfig {
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub fps: u32,
    pub degrees_per_second: f64,
    pub spacing: f64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            width: None,
            height: None,
            fps: 100,
            degrees_per_second: 60.0,
            spacing: 1.3,
        }
    }
}

impl AppConfig {
    pub fn frame_time(&self) -> Duration {
        Duration::from_secs_f64(1.0 / self.fps as f64)
    }

    /// Explicit config wins, else terminal size, else the fallback.
    pub fn canvas_dims(&self) -> (usize, usize) {
        let (tw, th) = terminal_size::terminal_size()
            .map(|(w, h)| (w.0 as usize, h.0 as usize))
            .unwrap_or((100, 40));

        (
            self.width.unwrap_or(tw),
            self.height.unwrap_or(th.saturating_sub(1)),
        )
    }
}

/// Mesh::cube() puts its vertices at +/-0.5 on each axis, so its corners sit
/// sqrt(3)/2 from the origin rather than 0.5 like every other mesh. Scale it
/// by 1/sqrt(3) so all five shapes sweep the same bounding sphere.
const CUBE_SCALE: f64 = 0.5773502692;

/// Per-object (spin axis, speed multiplier). Axes are non-symmetric so no
/// shape spins about one of its own symmetry axes and appears to snap, and
/// the rates differ so the five never tumble in lockstep.
const SPINS: [([f64; 3], f64); 5] = [
    ([0.30, 1.00, 0.15], 1.00),
    ([1.00, 0.35, 0.20], 0.75),
    ([0.20, 0.80, 1.00], 1.30),
    ([0.65, 0.25, 1.00], 0.90),
    ([1.00, 0.90, 0.40], 1.15),
];

/**
 * Transform placing an object on the X axis at the world origin's height,
 * with rotation left at identity — the animation loop overwrites it each
 * frame.
 */
fn placed_at(x: f64, scale: f64) -> Transform {
    Transform {
        translation: Vec3::new(x, 0.0, 0.0),
        scale: Vec3::ONE * scale,
        ..Default::default()
    }
}

fn main() -> io::Result<()> {
    let cfg = AppConfig::default();

    let (width, height) = cfg.canvas_dims();

    // Setup front / back buffers and terminal presenter
    let mut presenter = TerminalPresenter::new(width, height);

    let mut out = io::stdout().lock();

    let renderer = WireframeRenderer {
        ..Default::default()
    };

    // Sits on +Z looking down -Z (identity rotation) toward the origin.
    // At this distance the frustum is ~3.6 units wide either side of center,
    // so the outermost shapes (2.6 + 0.5 = 3.1) clear the edge with room left.
    let camera = Camera {
        position: Vec3::new(0.0, 0.0, 5.0),
        ..Default::default()
    };

    // Laid out left to right in increasing order of edge count
    let mut objects = vec![
        Object::new(
            Mesh::cube(),
            placed_at(-2.0 * cfg.spacing, CUBE_SCALE),
        ),
        Object::new(
            Mesh::tetrahedron(),
            placed_at(-1.0 * cfg.spacing, 1.0),
        ),
        Object::new(
            Mesh::octahedron(),
            placed_at(0.0, 1.0),
        ),
        Object::new(
            Mesh::icosahedron(),
            placed_at(1.0 * cfg.spacing, 1.0),
        ),
        Object::new(
            Mesh::dodecahedron(),
            placed_at(2.0 * cfg.spacing, 1.0),
        ),
    ];

    let frame_time = cfg.frame_time();
    let mut angle: f64 = 0.0;
    let mut last = Instant::now();

    term::enter_fullscreen(&mut out)?;

    loop {
        let frame_start = Instant::now();
        let dt = frame_start.duration_since(last).as_secs_f64();
        last = frame_start;

        angle = (angle + cfg.degrees_per_second * dt) % 360.0;

        // Rebuild each rotation from the running angle rather than
        // accumulating quaternion products, which would drift
        for (object, (axis, speed)) in objects.iter_mut().zip(SPINS) {
            object.transform.rotation = Quaternion::from_axis_angle(
                    Vec3::new(axis[0], axis[1], axis[2]),
                    angle * speed,
            );
        }

        {
            let buffer = presenter.begin_frame();
            renderer.render(
                &objects,
                &camera,
                WireframeStyle::default(),
                buffer,
            );
        }

        presenter.present(&mut out)?;
        thread::sleep(frame_time.saturating_sub(frame_start.elapsed()));
    }
}
