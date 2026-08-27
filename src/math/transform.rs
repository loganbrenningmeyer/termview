use super::{Mat4, Quaternion, Vec3};


#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quaternion,
    pub scale: Vec3,
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

