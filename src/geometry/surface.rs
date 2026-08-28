use super::{Edge, Mesh, Vertex};


impl Mesh {
    /**
     * Creates a wireframe mesh of a 3D surface given the 
     * (x, y) bounds and num of samples, with the 
     * corresponding z = f(x, y) function, where
     * -- F: fn(f64, f64) -> f64
     */
    pub fn surface<F>(
        x_min: f64,
        x_max: f64,
        y_min: f64,
        y_max: f64,
        x_samples: usize,
        y_samples: usize,
        function: F,
    ) -> Mesh 
    where 
        F: Fn(f64, f64) -> f64,
    {
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut edges: Vec<Edge> = Vec::new(); 

        // Track that values are finite
        let mut finite_idxs: Vec<Option<usize>> = 
            vec![None; x_samples * y_samples];

        // Iterate through each grid index (x_i, y_i)
        let x_range = x_max - x_min;
        let y_range = y_max - y_min;

        // Change in x / y per grid index
        let dx = x_range / (x_samples - 1) as f64;
        let dy = y_range / (y_samples - 1) as f64;

        for x_i in 0..x_samples {
            let x = x_min + dx * x_i as f64;

            for y_i in 0..y_samples {
                let y = y_min + dy * y_i as f64;
                let z = function(x, y);

                // Logical x/y grid index
                let grid_idx = x_i * y_samples + y_i;

                // NaN, +/-infinity
                if !z.is_finite() {
                    continue;
                }

                // Actual index into vertices, ignoring skipped non-finite z's
                let v_idx = vertices.len();

                vertices.push(Vertex::new(x, y, z));

                // Remember grid idx -> vertex idx
                finite_idxs[grid_idx] = Some(v_idx);

                // Connect to preceding y vertex (if exists and was finite)
                // -- Back single index
                if y_i > 0 {
                    let prev_grid_idx = grid_idx - 1;

                    if let Some(prev_v_idx) = finite_idxs[prev_grid_idx] {
                        edges.push(Edge::new(prev_v_idx, v_idx));
                    }
                }

                // Connect to adjacent x vertex (if exists and was finite)
                // -- Back one y line ago (y samples)
                if x_i > 0 {
                    let prev_grid_idx = grid_idx - y_samples;

                    if let Some(prev_v_idx) = finite_idxs[prev_grid_idx] {
                        edges.push(Edge::new(prev_v_idx, v_idx));
                    }
                }
            }
        }

        Mesh {
            vertices,
            edges,
        }
    }
}