mod bitmap;
mod collada;
mod frustum;
mod mesh;

use crate::math::{Matrix4, Quaternion, Vector3, Vector4};
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

impl Into<Matrix4> for &PerspectiveProjection {
    #[inline]
    fn into(self) -> Matrix4 {
        let depth = self.near - self.far;
        let tan_fov = (self.fov / 2.0).tan();
        Matrix4([
            Vector4([1.0 / (tan_fov * self.aspect_ratio), 0.0, 0.0, 0.0]),
            Vector4([0.0, 1.0 / tan_fov, 0.0, 0.0]),
            Vector4([0.0, 0.0, (self.near + self.far) / depth, -1.0]),
            Vector4([0.0, 0.0, (2.0 * self.far * self.near) / depth, 0.0]),
        ])
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Transform {
    pub position: Vector3,
    pub scale: Vector3,
    pub rotation: Quaternion,
}

impl Transform {
    #[inline]
    pub fn concat(&self, rhs: &Transform) -> Transform {
        Transform {
            position: self.position + rhs.position,
            scale: self.scale * rhs.scale,
            rotation: self.rotation * rhs.rotation,
        }
    }
}

impl Default for Transform {
    #[inline]
    fn default() -> Transform {
        Transform {
            position: Vector3::splat(0.0),
            scale: Vector3::splat(1.0),
            rotation: Quaternion::identity(),
        }
    }
}

impl Into<Matrix4> for &Transform {
    #[inline]
    fn into(self) -> Matrix4 {
        &(&Matrix4::scale(self.scale) * &self.rotation.normalized().into())
            * &Matrix4::translate(self.position)
    }
}
