use crate::{
    geometry::{Object, Vertex}, 
    math::{Mat4, Transform, Vec3, Vec4},
};
use super::{
    BrailleBuffer,
    Buffer, 
    Camera, 
    Cell,
};
use super::{
    draw_line,
};

#[derive(Debug, Clone, Copy)]
pub struct ScreenPoint {
    pub x: isize,
    pub y: isize,
    pub depth: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct WireframeStyle {
    pub edge: Cell,
    pub vertex: Cell,
}

impl Default for WireframeStyle {
    fn default() -> Self {
        Self {
            edge: Cell::new('#'),
            vertex: Cell::new('@'),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WireframeRenderer;

/**
 * Coordinates rendering pipeline for Objects and
 * their component vertices / edges
 * 
 * for each vertex
 *      object-space → world-space
 *      world-space  → camera-space
 *      camera-space → 2D plane
 *      2D coords    → terminal coords
 * 
 * for each edge
 *      draw line between projected endpoints
 */
impl WireframeRenderer {
    /**
     * Frame entry point. Compute camera view matrix per-frame,
     * not per-object. 
     * - Computes model-view-projection matrices for each object
     *      MVP = projection * view * model (reverse application)
     * - Projects vertices onto 2D screen
     * - Draws vertices / edges onto buffer
     */
    pub fn render(
        &self,
        objects: &[Object],
        camera: &Camera,
        style: WireframeStyle,
        buffer: &mut Buffer,
    ) {
        // Define perspective projection matrix / view matrix
        let proj = camera.projection.matrix(buffer.display_aspect());

        // [model matrix...] -> view matrix -> projection matrix
        let view = camera.view();

        for object in objects {
            // Projection * View * Model == Model → View → Projection
            let mvp = proj * view * object.transform.matrix();

            // Project vertex coordinates onto screen
            let vertices = &object.mesh.vertices;

            let proj_verts: Vec<Option<ScreenPoint>> = vertices
                .iter()
                .map(|v| self.project_vertex(
                    v, 
                    &mvp, 
                    buffer,
                ))
                .collect();

            // Render edges
            // - Ensure that vertices are Some() before render
            for edge in &object.mesh.edges {
                if let (Some(a), Some(b)) =
                    (proj_verts[edge.idx0], proj_verts[edge.idx1])
                {
                    self.render_edge(&a, &b, style, buffer);
                }
            }

            // Set vertex characters on Canvas
            for p in proj_verts.iter().flatten() {
                buffer.set(p.x, p.y, style.vertex);
            }
        }
    }

    /**
     * Render wireframe geometry into a 2x4 Braille subpixel grid,
     * then pack that grid into terminal cells.
     */
    pub fn render_braille(
        &self,
        objects: &[Object],
        camera: &Camera,
        style: WireframeStyle,
        buffer: &mut Buffer,
    ) {
        let display_aspect = buffer.display_aspect();
        let mut braille = BrailleBuffer::new(buffer.width(), buffer.height());

        self.render_braille_into(
            objects,
            camera,
            style,
            display_aspect,
            0,
            &mut braille,
        );

        braille.composite(buffer);
    }

    pub(crate) fn render_braille_into(
        &self,
        objects: &[Object],
        camera: &Camera,
        style: WireframeStyle,
        display_aspect: f64,
        layer: u8,
        braille: &mut BrailleBuffer,
    ) {
        let proj = camera.projection.matrix(display_aspect);
        let view = camera.view();

        for object in objects {
            let mvp = proj * view * object.transform.matrix();

            let projected: Vec<Option<ScreenPoint>> = object
                .mesh
                .vertices
                .iter()
                .map(|vertex| {
                    self.mvp_ndc_to_screen_dimensions(
                        vertex.position,
                        &mvp,
                        braille.width(),
                        braille.height(),
                    )
                })
                .collect();

            for edge in &object.mesh.edges {
                if let (Some(a), Some(b)) =
                    (projected[edge.idx0], projected[edge.idx1])
                {
                    braille.draw_line_depth(
                        a.x,
                        a.y,
                        a.depth,
                        b.x,
                        b.y,
                        b.depth,
                        layer,
                        style.edge.fg,
                    );
                }
            }

            for point in projected.iter().flatten() {
                braille.set_depth(
                    point.x,
                    point.y,
                    point.depth,
                    layer,
                    style.vertex.fg,
                );
            }
        }
    }

    /**
     * Project local object coord (x_obj, y_obj, z_obj) into a 
     * terminal coordinate based on the camera view and the object's
     * own transform
     */
    pub fn project_object_point(
        &self,
        point: Vec3,
        transform: Transform,
        camera: &Camera,
        buffer: &Buffer,
    ) -> Option<ScreenPoint> {
        // Create model-view-projection matrix
        let proj: Mat4  = camera.projection.matrix(buffer.display_aspect());
        let view: Mat4  = camera.view();
        let model: Mat4 = transform.matrix();

        let mvp = proj * view * model;

        self.mvp_ndc_to_screen(point, &mvp, buffer)
    }

    /**
     * Projects (x, y, z) world coordinate into a terminal
     * coordinate based on the camera view
     */
    pub fn project_world_point(
        &self,
        point: Vec3,
        camera: &Camera,
        buffer: &Buffer,
    ) -> Option<ScreenPoint> {
        // Define perspective projection matrix / view matrix
        let proj = camera.projection.matrix(buffer.display_aspect());

        // [model matrix...] -> view matrix -> projection matrix
        let mvp = proj * camera.view();

        self.mvp_ndc_to_screen(point, &mvp, buffer)
    }

    /**
     * Projects vertex from:
     * - object-space → clip-space → NDC → terminal cell
     * 
     * Returns None if the vertex can't be projected
     * - Behind the eye or outside near/far
     */
    fn project_vertex(
        &self,
        vertex: &Vertex,
        mvp: &Mat4,
        buffer: &Buffer,
    ) -> Option<ScreenPoint> {
        self.mvp_ndc_to_screen(vertex.position, mvp, buffer)
    }

    /**
     * 1. Given model-view-projection matrix, transforms point
     *    from local object-space to clip-space
     * 2. Converts from clip-space to normalized device coords (NDC)
     * 3. Converts from NDC to screen coordinates based on buffer dims
     */
    fn mvp_ndc_to_screen(
        &self,
        point: Vec3,
        mvp: &Mat4,
        buffer: &Buffer,
    ) -> Option<ScreenPoint> {
        self.mvp_ndc_to_screen_dimensions(
            point,
            mvp,
            buffer.width(),
            buffer.height(),
        )
    }

    fn mvp_ndc_to_screen_dimensions(
        &self,
        point: Vec3,
        mvp: &Mat4,
        width: usize,
        height: usize,
    ) -> Option<ScreenPoint> {
        if width == 0 || height == 0 {
            return None;
        }

        // Ignore non-finite points
        if !point.x.is_finite()
            || !point.y.is_finite()
            || !point.z.is_finite()
        {
            return None;
        }
        
        // Transform object-space → clip-space
        let clip = *mvp * Vec4 {
            x: point.x,
            y: point.y,
            z: point.z,
            w: 1.0,
        };

        // w == -z_view, so w <= 0 means at or behind the eye
        if clip.w <= f64::EPSILON {
            return None;
        }

        // clip-space → Normalized Device Coordinates (NDC)
        // - Apply perspective divide
        let ndc_x = clip.x / clip.w;
        let ndc_y = clip.y / clip.w;
        let ndc_z = clip.z / clip.w;

        // Ensure that NDC z-depth in normalized [-1, 1] range
        if !(-1.0..=1.0).contains(&ndc_z) {
            return None;    // outside near/far planes
        }

        // [-1, 1] NDC → [0, 1]
        // - Flip y, y-coords go bottom-top while rows go top-bottom
        let ndc_x_norm = (ndc_x + 1.0) * 0.5;
        let ndc_y_norm = (1.0 - ndc_y) * 0.5;

        // [0, 1] NDC to screen coordinates [0, dim)
        let screen_x = ndc_x_norm * (width - 1) as f64;
        let screen_y = ndc_y_norm * (height - 1) as f64;

        Some(ScreenPoint {
            x: screen_x.floor() as isize,
            y: screen_y.floor() as isize,
            depth: ndc_z,
        })
    }

    /**
     * Render edge between two ScreenPoints onto Buffer
     */
    fn render_edge(
        &self,
        a: &ScreenPoint,
        b: &ScreenPoint,
        style: WireframeStyle,
        buffer: &mut Buffer,
    ) {
        draw_line(
            buffer,
            a.x, 
            a.y, 
            b.x,
            b.y, 
            style.edge,
        );
    }
}
