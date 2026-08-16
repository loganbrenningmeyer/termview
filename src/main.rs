use std::time::Instant;
use std::thread;
use std::io::self;

mod config;
use config::AppConfig;

use termview::{
    camera::Camera,
    canvas::Canvas,
    geometry::{Mesh, Object},
    math::{Quaternion, Transform, Vec3},
    renderer::Renderer,
    term,
};

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
 * frame. All three Transform fields are Option, so each needs Some().
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

    // Setup Canvas / print buffer
    let mut canvas = Canvas::new(width, height);
    let mut frame = String::with_capacity((width + 1) * height);

    let mut out = io::stdout().lock();

    let renderer = Renderer {
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

        canvas.clear();
        renderer.render(&objects, &camera, &mut canvas);

        canvas.write_to(&mut frame);
        term::present(&mut out, &frame)?;
        thread::sleep(frame_time.saturating_sub(frame_start.elapsed()));
    }
}
