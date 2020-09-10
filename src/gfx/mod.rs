mod bitmap;
mod collada;
mod frustum;
mod mesh;

use crate::math::{Matrix4, Quaternion, Vector3};
pub use bitmap::*;
pub use collada::*;
pub use frustum::*;
pub use mesh::*;

#[derive(Default, Debug)]
pub struct PerspectiveProjection {
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: Vector3,
    pub scale: Vector3,
    pub rotation: Quaternion,
}

impl Transform {
    pub fn concat(&self, rhs: &Transform) -> Transform {
        Transform {
            position: self.position + rhs.position,
            scale: self.scale * rhs.scale,
            rotation: self.rotation * rhs.rotation,
        }
    }
}

impl Default for Transform {
    fn default() -> Transform {
        Transform {
            position: Vector3::splat(0.0),
            scale: Vector3::splat(1.0),
            rotation: Quaternion::identity(),
        }
    }
}

impl Into<Matrix4> for &Transform {
    fn into(self) -> Matrix4 {
        &(&Matrix4::scale(self.scale) * &self.rotation.normalized().into())
            * &Matrix4::translate(self.position)
    }
}
