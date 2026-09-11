use crate::math::{
    Mat4, 
    Projection, 
    Quaternion, 
    Vec3,
};

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: Vec3,
    pub rotation: Quaternion,
    pub projection: Projection,
    pub orbit: CameraOrbit,
}

/**
 * Camera representing the view from which world space points are projected
 * 
 * ```text
 *      position (Vec3): camera position in world coordinates
 *      rotation (Quaternion): camera local orientation relative to the world coordinate system
 *          - Represents camera accumulated rotation from its original orientation
 *          - current orientation = rotation * original orientation
 *          - After rotating the camera, the Quaternion describes where the original camera-local
 *            axes now point in world space
 * 
 *              let forward_world = camera.rotation.rotate_vec3(
 *                  Vec3::new(0.0, 0.0, -1.0)
 *              )
 * 
 *              // Gives the camera forward direction in world space
 * 
 *          - Applying rotation to a local direction gives that direction in world coordinates
 *          - Applying inverse rotation to a world-relative direction gives that direction
 *            in camera-local coordinates
 * ```
 */
impl Camera {
    pub const fn new(
        position: Vec3,
        rotation: Quaternion,
        projection: Projection,
        orbit: CameraOrbit,
    ) -> Self {
        Camera { position, rotation, projection, orbit }
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
     * Update camera position / rotation given azimuth, elevation, and distance,
     * oriented with +Z up
     */
    pub fn update(&mut self) {
        // Maps camera-local axes as follows:
        // right (+X)    -> world +Y
        // up (+Y)       -> world +Z
        // backward (+Z) -> world +X
        let z_up_basis = Quaternion::from_axis_angle(
            Vec3::new(1.0, 1.0, 1.0),
            120.0,
        );

        let rotation =
            Quaternion::from_axis_angle(Vec3::Z, self.orbit.azimuth)
            * z_up_basis
            * Quaternion::from_axis_angle(Vec3::X, -self.orbit.elevation);

        self.rotation = rotation;

        // The camera looks down local -Z, so placing it along its rotated
        // local +Z keeps it looking toward the origin.
        self.position =
            rotation.rotate_vec3(Vec3::Z * self.orbit.distance);
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
    fn default() -> Self {
        let mut camera = Self {
            position: Vec3::new(0.0, 0.0, 0.0),
            rotation: Quaternion::IDENTITY,
            projection: Projection::default(),
            orbit: CameraOrbit::default(),
        };

        camera.update();

        camera
    }
}


#[derive(Debug, Clone)]
pub struct CameraOrbit {
    pub azimuth: f64,
    pub elevation: f64,
    pub distance: f64,
}

impl Default for CameraOrbit {
    fn default() -> Self {
        Self {
            azimuth: 45.0,
            elevation: 25.0,
            distance: 10.0,
        }
    }
}
