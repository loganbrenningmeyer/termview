use crate::math::Mat4;

pub struct PerspectiveProjection {
    pub fov_y: f64,
    pub aspect: f64,
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
    pub fn matrix(&self) -> Mat4 {
        let mut mat = Mat4 {
            data: [0.0; 16],
        };

        // x': ==> x' = \text{focal} \cdot x / \text{aspect}
        let half_fov = (self.fov_y / 2.0).to_radians();
        let focal_length = 1.0 / half_fov.tan();

        mat.set(0, 0, focal_length / self.aspect);
        
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