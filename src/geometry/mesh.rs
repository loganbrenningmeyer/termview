use crate::math::Vec3;
use super::{Edge, Vertex};


pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub edges: Vec<Edge>,
}

impl Mesh {
    /**
     * Center on the bounding-box midpoint and scale to
     * circumradius 0.5
     */
    pub fn normalize(&mut self) {
        let Some(first) = self.vertices.first() else { return };

        // Determine the (x, y, z) min / max bounds
        let (mut min, mut max) = (first.position, first.position);

        for v in &self.vertices {
            min.x = min.x.min(v.position.x);
            min.y = min.y.min(v.position.y);
            min.z = min.z.min(v.position.z);
            max.x = max.x.max(v.position.x);
            max.y = max.y.max(v.position.y);
            max.z = max.z.max(v.position.z);
        }

        // Define the center (x, y, z) coords
        let center: Vec3 = (min + max) * 0.5;

        // Get maximum vertex radius magnitude
        let radius = self.vertices
            .iter()
            .map(|v| (v.position - center).magnitude())
            .fold(0.0_f64, f64::max);   // starts at 0, max() each vertex

        // Determine scale to get from max radius to 0.5 radius
        let scale = 0.5 / radius;

        for v in &mut self.vertices {
            v.position = (v.position - center) * scale;
        }
    }
}