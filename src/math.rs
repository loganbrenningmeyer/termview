use std::ops::{Add, Mul, Sub};

#[derive(Debug, Clone, Copy)]
pub enum Axes {
    X,
    Y,
    Z,
}

#[derive(Debug, Clone, Copy)]
pub struct Vec4 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Mat4 {
    pub data: [f64; 16],
}

#[derive(Debug, Clone, Copy)]
pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quaternion,
    pub scale: Vec3,
}

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0 );
    pub const ONE:  Self = Self::new(1.0, 1.0, 1.0 );
    pub const X:     Self = Self::new(1.0, 0.0, 0.0);
    pub const Y:     Self = Self::new(0.0, 1.0, 0.0);
    pub const Z:     Self = Self::new(0.0, 0.0, 1.0);
    pub const NEG_Z: Self = Self::new(0.0, 0.0, -1.0);

    pub const fn new(
        x: f64,
        y: f64,
        z: f64,
    ) -> Self {
        Self {x, y, z}
    }

    pub fn dot(self, rhs: Self) -> f64 {
        self.x * rhs.x +
        self.y * rhs.y +
        self.z * rhs.z
    }

    pub fn cross(self, rhs: Self) -> Vec3 {
        Vec3 {
            x: self.y * rhs.z - self.z * rhs.y,
            y: self.z * rhs.x - self.x * rhs.z,
            z: self.x * rhs.y - self.y * rhs.x,
        }
    }

    pub fn magnitude(self) -> f64 {
        self.dot(self).sqrt()
    }

    pub fn normalize(self) -> Vec3 {
        let mag = self.magnitude();

        Vec3 {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }
}

/**
 * Vec3 * scalar implementation overriding * operator
 */
impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: f64) -> Vec3 {
        Vec3 {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

/**
 * Vec3 + Vec3 implementation overriding + operator
 */
impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

/**
 * Vec3 - Vec3 implementation overriding - operator
 */
impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
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

/**
 * Create a Transform matrix given sequential translation,
 * rotation, and scaling
 * -- Apply in order: Scale -> Rotate -> Translate
 * 
 * 1. Scale the object around its local origin
 * 2. Rotate the object around its local origin
 * 3. Translate the object into world space
 */
impl Transform {
    pub const IDENTITY: Self = Self::new(
        Vec3::ZERO,
        Quaternion::IDENTITY,
        Vec3::ONE,
    );

    pub const fn new(
        translation: Vec3,
        rotation: Quaternion,
        scale: Vec3,
    ) -> Self {
        Transform {
            translation,
            rotation,
            scale,
        }
    }

    /**
     * Creates transformation model matrix for converting from
     * local coordinates to world coordinates
     * 
     * ==> M = TRS
     * ==> p_\text{world} = M p_\text{local}
     * 
     * Example:
     * - local object vertex coords: (1, 1, 1)
     * - object position: (10, 0, -5)
     * - world coords (no rot/scale): (11, 1, -4)
     */
    pub fn matrix(&self) -> Mat4 {
        let mat_trans = Mat4::translation_matrix(
            self.translation.x, 
            self.translation.y, 
            self.translation.z,
        );

        // Convert Quaternion to Mat4 rotation matrix
        let mat_rot = self.rotation.to_rotation_matrix();

        let mat_scale = Mat4::scaling_matrix(
            self.scale.x, 
            self.scale.y, 
            self.scale.z,
        );

        // Applied right -> left because matrix multiplication
        // is implemented using column vectors
        // -- mat_trans = T (R (S v)) 
        mat_trans * mat_rot * mat_scale
    }
}

impl Default for Transform {
    fn default() -> Self { Self::IDENTITY }
}

/**
 * Rotation quaternion for applying rotations to points
 * and storing current orientation
 */
impl Quaternion {
    pub const IDENTITY: Self = Self::new(1.0, 0.0, 0.0, 0.0);

    pub const fn new(
        w: f64,
        x: f64,
        y: f64,
        z: f64,
    ) -> Self {
        Quaternion {
            w: w,
            x: x,
            y: y,
            z: z,
        }
    }

    /**
     * Compute dot product between two Quaternions
     */
    pub fn dot(self, rhs: Self) -> f64 {
        self.w * rhs.w +
        self.x * rhs.x +
        self.y * rhs.y +
        self.z * rhs.z
    }

    /**
     * Compute Quaternion magnitude
     */
    pub fn magnitude(self) -> f64 {
        self.dot(self).sqrt()
    }

    /**
     * Normalize Quaternion values to 1 magnitude
     */
    pub fn normalize(self) -> Self {
        let mag = self.magnitude();

        Quaternion {
            w: self.w / mag,
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        }
    }

    /**
     * Compute the conjugate of a unit-normalized Quaternion
     * ==> q^* = (w, -x, -y, -z)
     */
    pub fn conjugate(self) -> Self {
        Quaternion {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    /**
     * Compute the inverse of a Quaternion
     * -- Unit-normalize first, then compute conjugate
     */
    pub fn inverse(self) -> Self {
        let q_norm = self.normalize();

        q_norm.conjugate()
    }

    /**
     * Creates Quaternion from axis of rotation and
     * angle of rotation in degrees
     * ==> q = \biggl (\cos \frac{\theta}{2}, a_x \sin \frac{\theta}{2}, a_y \sin \frac{\theta}{2}, a_z \sin \frac{\theta}{2} \biggr )
     */
    pub fn from_axis_angle(
        axis: Vec3,
        angle: f64,
    ) -> Self {
        let axis_norm = axis.normalize();
        let half_angle = angle / 2.0;
        
        let cos_half = half_angle.to_radians().cos();
        let sin_half = half_angle.to_radians().sin();

        Quaternion {
            w: cos_half,
            x: axis_norm.x * sin_half,
            y: axis_norm.y * sin_half,
            z: axis_norm.z * sin_half,
        }
    }

    /**
     * Converts Quaternion to Mat4 rotation matrix
     * 
     * -- First, to 3x3 rotation matrix:
     * 
     *  R_3 = [
     *      1 - 2(y^2 + z^2)  &  2(xy - wz)        &  2(xz + wy)
     *      2(xy + wz)        &  1 - 2(x^2 + z^2)  &  2(yz - wx)
     *      2(xz - wy)        &  2(yz + wx)        &  1 - 2(x^2 + y^2)
     *  ]
     * 
     * -- Then, put 3x3 rotation matrix in upper-left of Mat4 identity
     * 
     *  R_4 = [
     *      R_{00}  &  R_{01}  &  R_{02}  &  0
     *      R_{10}  &  R_{11}  &  R_{12}  &  0
     *      R_{20}  &  R_{21}  &  R_{22}  &  0
     *        0     &    0     &    0     &  1
     *  ]
     */
    pub fn to_rotation_matrix(self) -> Mat4 {
        let mut mat = Mat4::identity();

        let q = self.normalize();

        mat.set(0, 0, 1.0 - 2.0 * (q.y.powi(2) + q.z.powi(2)));
        mat.set(0, 1, 2.0 * (q.x * q.y - q.w * q.z));
        mat.set(0, 2, 2.0 * (q.x * q.z + q.w * q.y));

        mat.set(1, 0, 2.0 * (q.x * q.y + q. w * q.z));
        mat.set(1, 1, 1.0 - 2.0 * (q.x.powi(2) + q.z.powi(2)));
        mat.set(1, 2, 2.0 * (q.y * q.z - q.w * q.x));

        mat.set(2, 0, 2.0 * (q.x * q.z - q.w * q.y));
        mat.set(2, 1, 2.0 * (q.y * q.z + q.w * q.x));
        mat.set(2, 2, 1.0 - 2.0 * (q.x.powi(2) + q.y.powi(2)));

        mat
    }

    /**
     * Apply Quaternion rotation to Vec3 coordinates
     * ==> v' = q v_q q^{-1}
     */
    pub fn rotate_vec3(self, v: Vec3) -> Vec3 {
        let q = self.normalize();
        let q_inv = q.inverse();

        // Convert Vec3 to Quaternion (front-pad w = 0.0)
        let v_q = Quaternion::new(0.0, v.x, v.y, v.z);

        let v_rot = q * v_q * q_inv;

        Vec3::new(v_rot.x, v_rot.y, v_rot.z)
    }
}

/**
 * Quaternion * Quaternion overriding * operator
 * -- Computes the Hamilton product
 *
 * q1q2 = [ 
 *    w_1 w_2 - x_1 x_2 - y_1 y_2 - z_1 z_2  -> w
 *    w_1 x_2 + x_1 w_2 + y_1 z_2 - z_1 y_2  -> x
 *    w_1 y_2 - x_1 z_2 + y_1 w_2 + z_1 x_2  -> y
 *    w_1 z_2 + x_1 y_2 - y_1 x_2 + z_1 w_2  -> z
 * ]
 */
impl Mul<Quaternion> for Quaternion {
    type Output = Quaternion;

    fn mul(self, rhs: Quaternion) -> Quaternion {
        Quaternion {
            w: self.w*rhs.w - self.x*rhs.x - self.y*rhs.y - self.z*rhs.z,

            x: self.w*rhs.x + self.x*rhs.w + self.y*rhs.z - self.z*rhs.y,

            y: self.w*rhs.y - self.x*rhs.z + self.y*rhs.w + self.z*rhs.x,

            z: self.w*rhs.z + self.x*rhs.y - self.y*rhs.x + self.z*rhs.w,
        }
    }
}