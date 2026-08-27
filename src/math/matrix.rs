use std::ops::Mul;

use super::{Vec3, Vec4};


#[derive(Debug, Clone, Copy)]
pub enum Axes {
    X,
    Y,
    Z,
}


#[derive(Debug, Clone, Copy)]
pub struct Mat4 {
    pub data: [f64; 16],
}

/**
 * 4x4 matrix stored in row-major order, meaning
 * indices move across first row, then across second row, etc.:
 *
 * M   = [ m00 m01 m02 m03
 *         m10 m11 m12 m13
 *         m20 m21 m22 m23
 *         m30 m31 m32 m33 ]
 */
impl Mat4 {
    pub fn identity() -> Self {
        Self {
            data: [ 
                1.0, 0.0, 0.0, 0.0, 
                0.0, 1.0, 0.0, 0.0, 
                0.0, 0.0, 1.0, 0.0,
                0.0, 0.0, 0.0, 1.0,
            ],
        }
    }

    pub fn transpose(&self) -> Self {
        let mut mat_t = Mat4 {
            data: [0.0; 16],
        };

        for row in 0..4 {
            for col in 0..4 {
                mat_t.set(col, row, self.get(row, col));
            }
        }

        mat_t
    }

    /**
     * Creates translation matrix with offsets tx, ty, and tz
     */
    pub fn translation_matrix(
        tx: f64,
        ty: f64,
        tz: f64,
    ) -> Self {
        let mut mat_trans = Self::identity();

        mat_trans.set(0, 3, tx);
        mat_trans.set(1, 3, ty);
        mat_trans.set(2, 3, tz);
        
        mat_trans
    }

    /**
     * Creates rotation matrix given the axis of rotation and
     * the angle of rotation in degrees
     */
    pub fn rotation_matrix(
        axis: Axes,
        angle: f64,
    ) -> Self {
        let cos: f64 = angle.to_radians().cos();
        let sin: f64 = angle.to_radians().sin();

        let mut mat_rot = Self::identity();

        match axis {
            // X-axis rotation
            // | 1   0    0    0 |
            // | 0   cos -sin  0 |
            // | 0   sin  cos  0 |
            // | 0   0    0    1 |
            Axes::X => {
                mat_rot.set(1, 1,  cos);
                mat_rot.set(1, 2, -sin);
                mat_rot.set(2, 1,  sin);
                mat_rot.set(2, 2,  cos);
            },

            // Y-axis rotation
            // |  cos  0  sin  0 |
            // |   0   1   0   0 |
            // | -sin  0  cos  0 |
            // |   0   0   0   1 |
            Axes::Y => {
                mat_rot.set(0, 0,  cos);
                mat_rot.set(0, 2,  sin);
                mat_rot.set(2, 0, -sin);
                mat_rot.set(2, 2,  cos);
            },

            // Z-axis rotation
            // | cos -sin  0  0 |
            // | sin  cos  0  0 |
            // |  0    0   1  0 |
            // |  0    0   0  1 |
            Axes::Z => {
                mat_rot.set(0, 0,  cos);
                mat_rot.set(0, 1, -sin);
                mat_rot.set(1, 0,  sin);
                mat_rot.set(1, 1,  cos);
            },
        }

        mat_rot
    }

    /**
     * Creates scaling matrix given axes scales sx, sy, and sz
     */
    pub fn scaling_matrix(
        sx: f64,
        sy: f64,
        sz: f64,
    ) -> Self {
        let mut mat = Self::identity();

        mat.set(0, 0, sx);
        mat.set(1, 1, sy);
        mat.set(2, 2, sz);

        mat
    }

    /**
     * - Converts world-space point to camera-space point relative to
     *   the camera's position and target orientation
     */
    pub fn look_at(
        eye: Vec3,
        target: Vec3,
        world_up: Vec3,
    ) -> Mat4 {
        // Direction from camera to target
        let forward: Vec3 = (target - eye).normalize();
        // Relative right direction perpendicular to forward / world_up
        let right = forward.cross(world_up).normalize();
        // Camera's corrected up direction perpendicular to forward / right
        let up = right.cross(forward);

        let mut mat_view = Mat4::identity();

        mat_view.set(0, 0, right.x);
        mat_view.set(0, 1, right.y);
        mat_view.set(0, 2, right.z);
        mat_view.set(0, 3, -right.dot(eye));

        mat_view.set(1, 0, up.x);
        mat_view.set(1, 1, up.y);
        mat_view.set(1, 2, up.z);
        mat_view.set(1, 3, -up.dot(eye));

        mat_view.set(2, 0, -forward.x);
        mat_view.set(2, 1, -forward.y);
        mat_view.set(2, 2, -forward.z);
        mat_view.set(2, 3, forward.dot(eye));

        mat_view
    }

    /**
     * Given row / column, return the corresponding index
     * using row-major order
     * 
     * idx = row * 4 + col
     */
    fn index(row: usize, col: usize) -> usize {
        row * 4 + col
    }

    /**
     * Get a Mat4 value at given row / column position 
     */
    pub fn get(&self, row: usize, col: usize) -> f64 {
        let idx: usize = Self::index(row, col);
        self.data[idx]
    }

    /**
     * Set a Mat4 value at given row / column position
     */
    pub fn set(
        &mut self, 
        row: usize, 
        col: usize,
        value: f64,
    ) {
        let idx: usize = Self::index(row, col);
        self.data[idx] = value;
    }
}

/**
 * Mat4 * Mat4 multiplication overriding * operator
 */
impl Mul<Mat4> for Mat4 {
    type Output = Mat4;

    fn mul(self, rhs: Mat4) -> Mat4 {
        let mut result = Mat4 {
            data: [0.0; 16],
        };

        for row in 0..4 {
            for col in 0..4 {
                let mut sum = 0.0;

                for k in 0..4 {
                    sum += self.get(row, k) * rhs.get(k, col);
                }

                result.set(row, col, sum);
            }
        }

        result
    }
}

/**
 * Mat4 * Vec4 multiplication overriding * operator
 * 
 * Applies Mat4 to Vec4 [ x, y, z, w ] vector
 * ==> M \in \mathbb{R}^{4 \times 4}, v \in \mathbb{R}^{4 \times 1}
 * ==> Mv \in \mathbb{R}^{4 \times 1}
 */
impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Vec4 {
        Vec4 {
            x: 
                self.get(0, 0) * rhs.x +
                self.get(0, 1) * rhs.y +
                self.get(0, 2) * rhs.z +
                self.get(0, 3) * rhs.w,
            y: 
                self.get(1, 0) * rhs.x +
                self.get(1, 1) * rhs.y +
                self.get(1, 2) * rhs.z +
                self.get(1, 3) * rhs.w,
            z: 
                self.get(2, 0) * rhs.x +
                self.get(2, 1) * rhs.y +
                self.get(2, 2) * rhs.z +
                self.get(2, 3) * rhs.w,
            w: 
                self.get(3, 0) * rhs.x +
                self.get(3, 1) * rhs.y +
                self.get(3, 2) * rhs.z +
                self.get(3, 3) * rhs.w,
        }
    }
}

/**
 * Mat4 * f64 multiplication overriding * operator
 */
impl Mul<f64> for Mat4 {
    type Output = Mat4;

    fn mul(mut self, rhs: f64) -> Mat4 {
        for value in &mut self.data {
            *value *= rhs;
        }

        self
    }
}