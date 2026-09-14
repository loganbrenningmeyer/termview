use termview::{
    math::{Projection, Vec3},
    rendering::{Buffer, WireframeRenderer},
    ui::PlotView3d,
};

#[test]
fn orthographic_fit_keeps_surface_and_viewport_depth_visible() {
    let buffer = Buffer::new(100, 45);
    let renderer = WireframeRenderer;

    for (azimuth, elevation) in [(45.0, 25.0), (135.0, -25.0)] {
        let mut view = PlotView3d::default();
        view.camera.projection = Projection::Orthographic(Default::default());
        view.camera.orbit.azimuth = azimuth;
        view.camera.orbit.elevation = elevation;
        view.fit_camera(&buffer, &[]);

        // Reproduce the missing front half of sin(x) * cos(y).
        for x in -10..=10 {
            for y in -10..=10 {
                let (x, y) = (x as f64, y as f64);
                let point = Vec3::new(x, y, x.sin() * y.cos());
                assert!(renderer.project_object_point(
                    point, view.transform, &view.camera, &buffer,
                ).is_some(), "surface point {point:?} was depth-clipped");
            }
        }

        let Projection::Orthographic(projection) = &view.camera.projection else {
            unreachable!();
        };
        for x in [-10.0, 10.0] {
            for y in [-10.0, 10.0] {
                for z in [-10.0, 10.0] {
                    let depth = -view.camera.world_point_to_view(Vec3::new(x, y, z)).z;
                    assert!(depth > projection.near && depth < projection.far);
                }
            }
        }
    }
}

#[test]
fn orthographic_depth_fit_preserves_centered_axis_fill() {
    for (width, height) in [(100, 45), (45, 100)] {
        let buffer = Buffer::new(width, height);
        let mut view = PlotView3d::default();
        view.camera.projection = Projection::Orthographic(Default::default());
        view.fit_camera(&buffer, &[]);

        let Projection::Orthographic(projection) = &view.camera.projection else {
            unreachable!();
        };
        let mut min_x = f64::INFINITY;
        let mut max_x = f64::NEG_INFINITY;
        let mut min_y = f64::INFINITY;
        let mut max_y = f64::NEG_INFINITY;
        for point in [
            Vec3::new(-10.0, -10.0, -10.0),
            Vec3::new(10.0, -10.0, -10.0),
            Vec3::new(-10.0, 10.0, -10.0),
            Vec3::new(-10.0, -10.0, 10.0),
        ] {
            let q = view.camera.world_point_to_view(point);
            let x = q.x / (projection.size * buffer.display_aspect());
            let y = q.y / projection.size;
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }

        assert!((min_x + max_x).abs() < 1e-10);
        assert!((min_y + max_y).abs() < 1e-10);
        assert!((max_x.max(max_y) - 0.9).abs() < 1e-10);
    }
}
