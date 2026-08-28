use std::io;
use termview::{
    geometry::{Edge, Mesh, Object, Vertex}, 
    math::{Quaternion, Transform, Vec3}, 
    rendering::{Camera, Cell, Color, WireframeRenderer, WireframeStyle}, 
    terminal::{self as term, TerminalPresenter},
};


/**
 * 3D axis, defined as an edge connected two vertices
 * from the origin to the axis endpoint
 */
fn axis(
    endpoint: Vec3,
    transform: Transform,
) -> Object {
    Object::new(
        Mesh {
            vertices: vec![
                Vertex::new(0.0, 0.0, 0.0),
                Vertex::new(endpoint.x, endpoint.y, endpoint.z),
            ],
            edges: vec![Edge::new(0, 1)],
        },
        transform,
    )
}

fn main() -> io::Result<()> {
    let transform = Transform {
        rotation:
            Quaternion::from_axis_angle(Vec3::Y, -45.0)
            * Quaternion::from_axis_angle(Vec3::X, 15.0)
            * Quaternion::from_axis_angle(Vec3::Z, -15.0),
        ..Default::default()
    };

    let axes = [
        (axis(Vec3::X * 1.5, transform), Color::Rgb(255, 80, 80)),
        (axis(Vec3::Y * 1.5, transform), Color::Rgb(80, 255, 120)),
        (axis(Vec3::Z * 1.5, transform), Color::Rgb(80, 140, 255)),
    ];

    let mut out = io::stdout().lock();

    let (width, height) = term::canvas_dims();
    let mut presenter = TerminalPresenter::new(width, height);
    let buffer = presenter.begin_frame();

    let camera = Camera {
        position: Vec3::new(0.0, 0.0, 3.0),
        ..Default::default()
    };
    let renderer = WireframeRenderer::default();

    for (axis, color) in &axes {
        let style = WireframeStyle {
            edge: Cell::new('━').with_fg(*color),
            vertex: Cell::new('●').with_fg(*color),
        };

        renderer.render(
            std::slice::from_ref(axis),
            &camera,
            style,
            buffer,
        );
    }

    let surface = Object::new(
        Mesh::surface(
            -1.0, 1.0,
            -1.0, 1.0,
            20, 20,
            |x, y| 0.5 * (x * x + y * y),
        ),
        transform,
    );

    renderer.render(
        std::slice::from_ref(&surface),
        &camera,
        WireframeStyle::default(),
        buffer,
    );

    term::enter_fullscreen(&mut out)?;

    presenter.present(&mut out)?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    term::leave_fullscreen(&mut out)?;

    Ok(())
}
