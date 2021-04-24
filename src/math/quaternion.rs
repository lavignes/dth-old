use crate::math::{Vector3, Vector4};
use std::ops::{Mul, MulAssign};

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Quaternion(pub Vector4);

unsafe impl bytemuck::Zeroable for Quaternion {}

unsafe impl bytemuck::Pod for Quaternion {}

/// Think of it like a unit vector with a 4th "twist" component.
impl Quaternion {
    #[inline]
    pub const fn identity() -> Quaternion {
        Quaternion(Vector4([0.0, 0.0, 0.0, 1.0]))
    }

    #[inline]
    pub fn from_axis_angle(axis: Vector3, angle: f32) -> Quaternion {
        let half_theta = angle / 2.0;
        let sin_half_theta = half_theta.sin();
        let cos_half_theta = half_theta.cos();
        Quaternion((axis * sin_half_theta).widened(cos_half_theta))
    }

    #[inline]
    pub fn from_angle_right(angle: f32) -> Quaternion {
        Quaternion::from_axis_angle(Vector3::right(), angle)
    }

    #[inline]
    pub fn from_angle_up(angle: f32) -> Quaternion {
        Quaternion::from_axis_angle(Vector3::up(), angle)
    }

    #[inline]
    pub fn from_angle_forward(angle: f32) -> Quaternion {
        Quaternion::from_axis_angle(Vector3::forward(), angle)
    }

    #[inline]
    pub fn normalized(&self) -> Quaternion {
        Quaternion(self.0.normalized())
    }

    #[inline]
    pub fn conjugated(&self) -> Quaternion {
        Quaternion(Vector4([-self.0[0], -self.0[1], -self.0[2], self.0[3]]))
    }

    #[inline]
    pub fn right_axis(&self) -> Vector3 {
        Vector3::right().rotated(*self)
    }

    #[inline]
    pub fn left_axis(&self) -> Vector3 {
        Vector3::left().rotated(*self)
    }

    #[inline]
    pub fn up_axis(&self) -> Vector3 {
        Vector3::up().rotated(*self)
    }

    #[inline]
    pub fn forward_axis(&self) -> Vector3 {
        Vector3::forward().rotated(*self)
    }

    /// A *very* fast interpolation. Only really useful for
    /// interpolating as long as the quaternions are aligned with
    /// an axis.
    pub fn lerp(&self, rhs: Quaternion, dt: f32) -> Quaternion {
        let cos_half_theta = self.0.dot(rhs.0);
        if cos_half_theta < 0.0 {
            return Quaternion(((-self.0) - rhs.0) * dt + self.0);
        }
        Quaternion((self.0 - rhs.0) * dt + self.0)
    }

    /// A fast interpolation. Only really useful for
    /// interpolating as long as the quaternions are aligned with
    /// an axis.
    #[inline]
    pub fn nlerp(&self, rhs: Quaternion, dt: f32) -> Quaternion {
        self.lerp(rhs, dt).normalized()
    }

    /// Interpolate between two quaternions.
    pub fn slerp(&self, rhs: Quaternion, dt: f32) -> Quaternion {
        let cos_half_theta = self.0.dot(rhs.0);
        if cos_half_theta.abs() >= 1.0 {
            return *self;
        }
        let sin_half_theta = (1.0 - cos_half_theta * cos_half_theta).sqrt();
        if sin_half_theta.abs() <= f32::EPSILON {
            return Quaternion(self.0 * 0.5 + rhs.0 * 0.5);
        }
        let half_theta = cos_half_theta.acos();
        let a = ((1.0 - dt) * half_theta).sin() / sin_half_theta;
        let b = (dt * half_theta).sin() / sin_half_theta;
        Quaternion(self.0 * a + rhs.0 * b)
    }
}

impl MulAssign<Quaternion> for Quaternion {
    #[inline]
    fn mul_assign(&mut self, rhs: Quaternion) {
        *self = *self * rhs;
    }
}

impl Mul<Quaternion> for Quaternion {
    type Output = Quaternion;
    #[rustfmt::skip]
    fn mul(self, rhs: Quaternion) -> Quaternion {
        Quaternion(Vector4([
            self.0[0] * rhs.0[3] + self.0[3] * rhs.0[0] + self.0[1] * rhs.0[2] - self.0[2] * rhs.0[1],
            self.0[1] * rhs.0[3] + self.0[3] * rhs.0[1] + self.0[2] * rhs.0[0] - self.0[0] * rhs.0[2],
            self.0[2] * rhs.0[3] + self.0[3] * rhs.0[2] + self.0[0] * rhs.0[1] - self.0[1] * rhs.0[0],
            self.0[3] * rhs.0[3] - self.0[0] * rhs.0[0] - self.0[1] * rhs.0[1] - self.0[2] * rhs.0[2],
        ]))
    }
}

impl Mul<Vector3> for Quaternion {
    type Output = Quaternion;
    fn mul(self, rhs: Vector3) -> Quaternion {
        Quaternion(Vector4([
            self.0[3] * rhs.0[0] + self.0[1] * rhs.0[2] - self.0[2] * rhs.0[1],
            self.0[3] * rhs.0[1] + self.0[2] * rhs.0[0] - self.0[0] * rhs.0[2],
            self.0[3] * rhs.0[2] + self.0[0] * rhs.0[1] - self.0[1] * rhs.0[0],
            -self.0[0] * rhs.0[0] - self.0[1] * rhs.0[1] - self.0[2] * rhs.0[2],
        ]))
    }
}
