use std::ops::Mul;

use super::{Mat4, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
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