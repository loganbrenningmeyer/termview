use std::fmt;
use crate::math::Mat4;

#[derive(Debug)]
pub enum Projection {
    Perspective(PerspectiveProjection),
    Orthographic(OrthographicProjection),
}

impl Projection {
    pub fn matrix(&self, aspect: f64) -> Mat4 {
        match self {
            Projection::Perspective(p) => p.matrix(aspect),
            Projection::Orthographic(o) => o.matrix(aspect),
        }
    }

    pub fn near(&self) -> f64 {
        match self {
            Projection::Perspective(p) => p.near,
            Projection::Orthographic(o) => o.near,
        }
    }
}

impl Default for Projection {
    fn default() -> Self {
        Projection::Perspective(PerspectiveProjection::default())
    }
}

impl fmt::Display for Projection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Projection::Perspective(_) => write!(f, "Perspective"),
            Projection::Orthographic(_) => write!(f, "Orthographic"),
        }
    }
}


#[derive(Debug)]
pub struct PerspectiveProjection {
    pub fov_y: f64,
    pub near: f64,
    pub far: f64,
}

impl PerspectiveProjection {
    /**
     * Perspective projection matrix
     * - Creates perspective matrix which projects vertices in 
     *   camera view space onto camera clip space
     * - Creates perspective effect; given camera view coords, how 
     *   would perspective projection affect the points?
     * 
     *      x       y       z                  |      w
     *                                         |
     * [ f/aspect   0       0                  |      0                    ]  -> x' = (f/aspect) * x
     * [    0       f       0                  |      0                    ]  -> y' = f * y
     * [    0       0   -(far+near)/(far-near) |  -(2·far·near)/(far-near) ]  -> z' = A * z + B
     * [    0       0      -1                  |      0                    ]  -> w' = -z
     * 
     * ==> p_\text{clip} = P p_\text{view}
     */
    pub fn matrix(&self, aspect: f64) -> Mat4 {
        let mut mat = Mat4 {
            data: [0.0; 16],
        };

        // x': ==> x' = \text{focal} \cdot x / \text{aspect}
        let half_fov = (self.fov_y / 2.0).to_radians();
        let focal_length = 1.0 / half_fov.tan();

        mat.set(0, 0, focal_length / aspect);
        
        // y': ==> y' = \text{focal} \cdot y
        mat.set(1, 1, focal_length);

        // z': ==> z' = \frac{-(f + n)}{f - n} \cdot z + \frac{-(2fn)}{f - n}
        let a = -(self.far + self.near) / (self.far - self.near);
        let b = -(2.0 * self.far * self.near) / (self.far - self.near);

        mat.set(2, 2, a);
        mat.set(2, 3, b);

        // w': ==> w' = -z
        mat.set(3, 2, -1.0);

        mat
    }
}

impl Default for PerspectiveProjection {
    fn default() -> Self {
        Self {
            fov_y: 50.0,
            near: 0.1,
            far: 100.0,
        }
    }
}


#[derive(Debug)]
pub struct OrthographicProjection {
    pub size: f64,
    pub near: f64,
    pub far: f64,
}

impl OrthographicProjection {
    /**
     * Orthographic projection matrix
     * - Similar to perspective projection but uses a box
     *   instead of a frustum with fov; all rays are parallel
     *   - Since w=1 in the last row, there is no division by depth
     * - Any ray within the box is rendered
     * 
     *     x        y        z      |        w
     * [ 2/(r-l)    0        0      |  -(r+l)/(r-l) ]  -> x'
     * [   0      2/t-b      0      |  -(t+b)/(t-b) ]  -> y'
     * [   0        0     -2/(f-n)  |  -(f+n)/(f-n) ]  -> z'
     * [   0        0        0      |        1      ]  -> w'
     */
    pub fn matrix(&self, aspect: f64) -> Mat4 {
        let mut mat = Mat4 {
            data: [0.0; 16],
        };

        let t = self.size;
        let b = -self.size;
        let r = self.size * aspect;
        let l = -self.size * aspect;

        let f = self.far;
        let n = self.near;

        // x'
        mat.set(0, 0, 2.0/(r - l));
        mat.set(0, 3, -(r + l)/(r - l));

        // y'
        mat.set(1, 1, 2.0/(t - b));
        mat.set(1, 3, -(t + b)/(t - b));

        // z'
        mat.set(2, 2, -2.0/(f - n));
        mat.set(2, 3, -(f + n)/(f - n));

        // w'
        mat.set(3, 3, 1.0);

        mat
    }
}

impl Default for OrthographicProjection {
    fn default() -> Self {
        Self {
            size: 5.0,
            near: 0.1,
            far: 100.0,
        }
    }
}