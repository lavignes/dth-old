use crate::{
    gfx::PerspectiveProjection,
    math::{Vector2, Vector3},
};

// This is based on the neat radar frustum culling approach on lighthouse3d
// http://www.lighthouse3d.com/tutorials/view-frustum-culling/
#[derive(Debug, Default)]
pub struct Frustum {
    position: Vector3,
    // I believe these are the unit axes of the "camera" (relative to its rotation)
    x: Vector3,
    y: Vector3,
    z: Vector3,

    // This is a fudge-factor needed for sphere-testing
    // AFAICT it is y-distance a sphere must be at a given point along the "camera"'s z-axis to be
    // considered in the frustum. (i.e. it tests if a sphere is inside the top and bottom
    // planes of the frustum) Using the aspect-ratio, we calculate it once.
    sphere_factor: Vector2,

    tan_fov: f32,
    aspect_ratio: f32,
    near: f32,
    far: f32,
}

impl Frustum {
    #[inline]
    pub fn new(
        projection: &PerspectiveProjection,
        position: Vector3,
        at: Vector3,
        up: Vector3,
    ) -> Frustum {
        let mut frustum = Frustum::default();
        frustum.update_projection(projection);
        frustum.update_look_at(position, at, up);
        frustum
    }

    pub fn update_projection(&mut self, projection: &PerspectiveProjection) {
        self.aspect_ratio = projection.aspect_ratio;
        self.near = projection.near;
        self.far = projection.far;
        self.tan_fov = projection.fov.tan();

        let fov_x = (self.tan_fov * projection.aspect_ratio).atan();
        self.sphere_factor = (1.0 / fov_x.cos(), 1.0 / projection.fov.cos()).into();
    }

    pub fn update_look_at(&mut self, position: Vector3, at: Vector3, up: Vector3) {
        self.position = position;
        self.z = (position - at).normalized();
        self.x = (up * self.z).normalized();
        self.y = self.z * self.x;
    }

    pub fn point_inside(&self, position: Vector3) -> bool {
        // vector from "camera" to position
        let to_position = position - self.position;

        // Find how far the z is and if its within the near and far planes
        let z = to_position.dot(-self.z);
        if z > self.far || z < self.near {
            return false;
        }

        // Find the width/2 (via height*aspect ratio) of the frustum at z and check if we're inside
        let x = to_position.dot(self.x);
        let half_width_at_z = z * self.tan_fov * self.aspect_ratio;
        if x > half_width_at_z || x < -half_width_at_z {
            return false;
        }

        // Find the height/2 of the frustum at z and check if we're inside
        let y = to_position.dot(self.y);
        let half_height_at_z = z * self.tan_fov;
        if y > half_height_at_z || y < -half_height_at_z {
            return false;
        }

        true
    }

    pub fn infinite_cylinder_inside(&self, position: Vector3, radius: f32) -> bool {
        // vector from "camera" to position
        let to_position = position - self.position;

        // Find how far the z is and if its within the near and far planes
        let z = to_position.dot(-self.z);
        if z > (self.far + radius) || z < (self.near - radius) {
            return false;
        }

        let test_distance = self.sphere_factor * radius;

        // Then y (using the sphere-factor)
        let y = to_position.dot(self.y);
        let half_height_at_z = z * self.tan_fov;
        if y > half_height_at_z + test_distance.y() || y < -half_height_at_z - test_distance.y() {
            return false;
        }

        true
    }

    pub fn sphere_inside(&self, position: Vector3, radius: f32) -> bool {
        // vector from "camera" to position
        let to_position = position - self.position;

        // Find how far the z is and if its within the near and far planes
        let z = to_position.dot(-self.z);
        if z > (self.far + radius) || z < (self.near - radius) {
            return false;
        }

        let test_distance = self.sphere_factor * radius;

        // Now for x (using the sphere-factor)
        let x = to_position.dot(self.x);
        let half_width_at_z = z * self.tan_fov * self.aspect_ratio;
        if x > half_width_at_z + test_distance.x() || x < -half_width_at_z - test_distance.x() {
            return false;
        }

        // Then y (using the sphere-factor)
        let y = to_position.dot(self.y);
        let half_height_at_z = z * self.tan_fov;
        if y > half_height_at_z + test_distance.y() || y < -half_height_at_z - test_distance.y() {
            return false;
        }

        true
    }
}
