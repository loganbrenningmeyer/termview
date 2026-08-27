use crate::{
    geometry::{Object, Vertex}, 
    math::{Mat4, Vec4},
};
use super::{
    Buffer, 
    Camera, 
    Cell,
    PerspectiveProjection
};
use super::{
    draw_line,
};

pub struct WireframeRenderer {
    pub fov_y: f64,
    pub near: f64,
    pub far: f64,
    pub cell_aspect: f64,
    pub edge_cell: Cell,
    pub vertex_cell: Cell,
}

#[derive(Debug, Clone, Copy)]
pub struct ScreenPoint {
    pub x: isize,
    pub y: isize,
    pub depth: f64,
}

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
        buffer: &mut Buffer,
    ) {
        // Define perspective projection matrix / view matrix
        let proj_mat = PerspectiveProjection {
            fov_y: self.fov_y,
            aspect: buffer.aspect() * self.cell_aspect,
            near: self.near,
            far: self.far,
        }.matrix();

        // [model matrix...] -> view matrix -> projection matrix
        let view_proj = proj_mat * camera.view();

        for object in objects {
            // Projection * View * Model == Model → View → Projection
            let mvp = view_proj * object.transform.matrix();

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
                    self.render_edge(&a, &b, buffer);
                }
            }

            // Set vertex characters on Canvas
            for p in proj_verts.iter().flatten() {
                buffer.set(p.x, p.y, self.vertex_cell);
            }
        }
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
        // Transform object-space → clip-space
        let clip = *mvp * Vec4 {
            x: vertex.position.x,
            y: vertex.position.y,
            z: vertex.position.z,
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

        // [0, 1] NDC to screen coordinates
        let screen_x = ndc_x_norm * buffer.width() as f64;
        let screen_y = ndc_y_norm * buffer.height() as f64;

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
        buffer: &mut Buffer,
    ) {
        draw_line(
            buffer,
            a.x, 
            a.y, 
            b.x,
            b.y, 
            self.edge_cell,
        );
    }
}

impl Default for WireframeRenderer {
    fn default() -> Self {
        WireframeRenderer {
            fov_y: 50.0,
            near: 0.1,
            far: 100.0,
            cell_aspect: 0.5,
            edge_cell: Cell { ch: '#' },
            vertex_cell: Cell { ch: '@' },
        }
    }
}
