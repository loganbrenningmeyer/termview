use crate::geometry::{Edge, Mesh, Vertex};

impl Mesh {
    pub fn cube() -> Self {
        let vertices = vec![
            Vertex::new(-0.5, -0.5, -0.5),
            Vertex::new( 0.5, -0.5, -0.5),
            Vertex::new( 0.5,  0.5, -0.5),
            Vertex::new(-0.5,  0.5, -0.5),
            Vertex::new(-0.5, -0.5,  0.5),
            Vertex::new( 0.5, -0.5,  0.5),
            Vertex::new( 0.5,  0.5,  0.5),
            Vertex::new(-0.5,  0.5,  0.5),
        ];

        let edges = vec![
            Edge::new(0, 1),
            Edge::new(1, 2),
            Edge::new(2, 3),
            Edge::new(3, 0),

            Edge::new(4, 5),
            Edge::new(5, 6),
            Edge::new(6, 7),
            Edge::new(7, 4),

            Edge::new(0, 4),
            Edge::new(1, 5),
            Edge::new(2, 6),
            Edge::new(3, 7),
        ];

        Mesh { 
            vertices, 
            edges,
        }
    }

    pub fn dodecahedron() -> Self {
        let phi = 1.61803398875;
        let inv_phi = 1.0 / phi;

        // Standard dodecahedron coordinates have radius sqrt(3).
        // Scale to a circumradius of 0.5.
        let s = 0.28867513459;

        let vertices = vec![
            // (±1, ±1, ±1)
            Vertex::new(-s, -s, -s), // 0
            Vertex::new(-s, -s,  s), // 1
            Vertex::new(-s,  s, -s), // 2
            Vertex::new(-s,  s,  s), // 3
            Vertex::new( s, -s, -s), // 4
            Vertex::new( s, -s,  s), // 5
            Vertex::new( s,  s, -s), // 6
            Vertex::new( s,  s,  s), // 7

            // (0, ±1/phi, ±phi)
            Vertex::new(0.0, -inv_phi * s, -phi * s), // 8
            Vertex::new(0.0, -inv_phi * s,  phi * s), // 9
            Vertex::new(0.0,  inv_phi * s, -phi * s), // 10
            Vertex::new(0.0,  inv_phi * s,  phi * s), // 11

            // (±1/phi, ±phi, 0)
            Vertex::new(-inv_phi * s, -phi * s, 0.0), // 12
            Vertex::new(-inv_phi * s,  phi * s, 0.0), // 13
            Vertex::new( inv_phi * s, -phi * s, 0.0), // 14
            Vertex::new( inv_phi * s,  phi * s, 0.0), // 15

            // (±phi, 0, ±1/phi)
            Vertex::new(-phi * s, 0.0, -inv_phi * s), // 16
            Vertex::new(-phi * s, 0.0,  inv_phi * s), // 17
            Vertex::new( phi * s, 0.0, -inv_phi * s), // 18
            Vertex::new( phi * s, 0.0,  inv_phi * s), // 19
        ];

        let edges = vec![
            Edge::new(0, 8),
            Edge::new(0, 12),
            Edge::new(0, 16),

            Edge::new(1, 9),
            Edge::new(1, 12),
            Edge::new(1, 17),

            Edge::new(2, 10),
            Edge::new(2, 13),
            Edge::new(2, 16),

            Edge::new(3, 11),
            Edge::new(3, 13),
            Edge::new(3, 17),

            Edge::new(4, 8),
            Edge::new(4, 14),
            Edge::new(4, 18),

            Edge::new(5, 9),
            Edge::new(5, 14),
            Edge::new(5, 19),

            Edge::new(6, 10),
            Edge::new(6, 15),
            Edge::new(6, 18),

            Edge::new(7, 11),
            Edge::new(7, 15),
            Edge::new(7, 19),

            Edge::new(8, 10),
            Edge::new(9, 11),

            Edge::new(12, 14),
            Edge::new(13, 15),

            Edge::new(16, 17),
            Edge::new(18, 19),
        ];

        Mesh {
            vertices,
            edges,
        }
    }

    pub fn icosahedron() -> Self {
        // Golden ratio.
        let phi = 1.61803398875;

        // Standard icosahedron coordinates have radius
        // sqrt(1 + phi^2). Scale that radius down to 0.5.
        let s = 0.26286555606;

        let vertices = vec![
            // (0, ±1, ±phi)
            Vertex::new(0.0, -s, -phi * s), // 0
            Vertex::new(0.0, -s,  phi * s), // 1
            Vertex::new(0.0,  s, -phi * s), // 2
            Vertex::new(0.0,  s,  phi * s), // 3

            // (±1, ±phi, 0)
            Vertex::new(-s, -phi * s, 0.0), // 4
            Vertex::new(-s,  phi * s, 0.0), // 5
            Vertex::new( s, -phi * s, 0.0), // 6
            Vertex::new( s,  phi * s, 0.0), // 7

            // (±phi, 0, ±1)
            Vertex::new(-phi * s, 0.0, -s), // 8
            Vertex::new(-phi * s, 0.0,  s), // 9
            Vertex::new( phi * s, 0.0, -s), // 10
            Vertex::new( phi * s, 0.0,  s), // 11
        ];

        let edges = vec![
            Edge::new(0, 2),
            Edge::new(0, 4),
            Edge::new(0, 6),
            Edge::new(0, 8),
            Edge::new(0, 10),

            Edge::new(1, 3),
            Edge::new(1, 4),
            Edge::new(1, 6),
            Edge::new(1, 9),
            Edge::new(1, 11),

            Edge::new(2, 5),
            Edge::new(2, 7),
            Edge::new(2, 8),
            Edge::new(2, 10),

            Edge::new(3, 5),
            Edge::new(3, 7),
            Edge::new(3, 9),
            Edge::new(3, 11),

            Edge::new(4, 6),
            Edge::new(4, 8),
            Edge::new(4, 9),

            Edge::new(5, 7),
            Edge::new(5, 8),
            Edge::new(5, 9),

            Edge::new(6, 10),
            Edge::new(6, 11),

            Edge::new(7, 10),
            Edge::new(7, 11),

            Edge::new(8, 9),
            Edge::new(10, 11),
        ];

        Mesh {
            vertices,
            edges,
        }
    }

    pub fn octahedron() -> Self {
        let vertices = vec![
            Vertex::new( 0.5,  0.0,  0.0), // 0 right
            Vertex::new(-0.5,  0.0,  0.0), // 1 left
            Vertex::new( 0.0,  0.5,  0.0), // 2 top
            Vertex::new( 0.0, -0.5,  0.0), // 3 bottom
            Vertex::new( 0.0,  0.0,  0.5), // 4 front
            Vertex::new( 0.0,  0.0, -0.5), // 5 back
        ];

        let edges = vec![
            // Top
            Edge::new(2, 0),
            Edge::new(2, 1),
            Edge::new(2, 4),
            Edge::new(2, 5),

            // Bottom
            Edge::new(3, 0),
            Edge::new(3, 1),
            Edge::new(3, 4),
            Edge::new(3, 5),

            // Equator
            Edge::new(0, 4),
            Edge::new(4, 1),
            Edge::new(1, 5),
            Edge::new(5, 0),
        ];

        Mesh {
            vertices,
            edges,
        }
    }

    pub fn tetrahedron() -> Self {
        // Scaled so every vertex is 0.5 units from the origin.
        let s = 0.28867513459;

        let vertices = vec![
            Vertex::new( s,  s,  s),
            Vertex::new( s, -s, -s),
            Vertex::new(-s,  s, -s),
            Vertex::new(-s, -s,  s),
        ];

        let edges = vec![
            Edge::new(0, 1),
            Edge::new(0, 2),
            Edge::new(0, 3),
            Edge::new(1, 2),
            Edge::new(1, 3),
            Edge::new(2, 3),
        ];

        Mesh {
            vertices,
            edges,
        }
    }
}