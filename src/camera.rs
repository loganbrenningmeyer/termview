use crate::math::{Mat4, Quaternion, Vec3};

pub struct Camera {
    pub position: Vec3,
    pub rotation: Quaternion,
}

/**
 * Camera representing the view from which world space points are projected
 * 
 *      position (Vec3): Camera position in world coordinates
 *      rotation (Quaternion): Camera's local orientation relative to the world coordinate system
 *          - Represents Camera's accumulated rotation from its original orientation
 *          - current orientation = rotation ⨯ original orientation
 *          - After rotating the camera, the Quaternion describes where the original camera-local
 *            axes now point in world space
 * 
 *              let forward_world = camera.rotation.rotate_vec3(
 *                  Vec3::new(0.0, 0.0, -1.0)
 *              )
 * 
 *              // Gives the camera's forward direction in world space
 * 
 *          - Applying rotation to a local direction gives that direction in world coordinates
 *          - Applying inverse rotation to a world-relative direction gives that direction
 *            in camera-local coordinates
 */
impl Camera {
    pub const ORIGIN: Self = Self::new(
        Vec3::new(0.0, 0.0, 0.0),
        Quaternion::IDENTITY,
    );

    pub const fn new(
        position: Vec3,
        rotation: Quaternion,
    ) -> Self {
        Camera { position, rotation }
    }

    /**
     * Construct Mat4 view matrix for converting world coordinates
     * to camera-local coordinates
     * 
     * 1. world point → subtract camera position = camera world pos
     * 2. camera world pos → inverse rotation = camera world pos / orientation
     * 
     * Translation first, followed by rotation as this reverses standard
     * scaling → rotation → translation order for world → camera coords (going in reverse)
     */
    pub fn view(&self) -> Mat4 {
        // Inverse rotation for world coords → camera coords
        let rotation = self.rotation.inverse().to_rotation_matrix();
        
        // Offset camera position back to world coords origin
        let translation = Mat4::translation_matrix(
            -self.position.x,
            -self.position.y,
            -self.position.z,
        );

        rotation * translation
    }

    /**
     * Project world coordinates to camera view coordinates,
     * i.e., 3D coordinates in the world coordinate system -> 3D coordinates relative to the camera's local origin
     */
    pub fn world_point_to_view(&self, point: Vec3) -> Vec3 {
        // Point relative to local camera coordinates
        let relative_point = point - self.position;
        // rotation converts local camera directions to world space,
        // must inverse for world space to camera space
        self.rotation.inverse().rotate_vec3(relative_point)
    }

    /**
     * Convert local camera coordinates to world coordinates,
     * i.e., 3D coordinates relative to the camera's local origin -> 3D coordinates in the world coordinate system
     */
    pub fn view_point_to_world(&self, point: Vec3) -> Vec3 {
        self.rotation.rotate_vec3(point) + self.position
    }

    /**
     * Get forward facing direction based on Quaternion
     * orientation and initial (0, 0, -1) facing in the -Z direction
     */
    pub fn forward(&self) -> Vec3 {
        self.rotation.rotate_vec3(Vec3::NEG_Z)
    }

    /**
     * Get right facing direction based on Quaternion
     * orientation and initial (0, 0, -1) facing in the -Z direction
     */
    pub fn right(&self) -> Vec3 {
        self.rotation.rotate_vec3(Vec3::X)
    }

    /**
     * Get up direction based on Quaternion orientation
     * and initial world-coords (0, 1, 0) upward direction
     */
    pub fn up(&self) -> Vec3 {
        self.rotation.rotate_vec3(Vec3::Y)
    }
}

impl Default for Camera {
    fn default() -> Self { Self::ORIGIN }
}