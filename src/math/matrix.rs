use crate::math::{Quaternion, Vector3, Vector4};

use crate::gfx::PerspectiveProjection;
use std::ops::{Index, IndexMut, Mul};

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Matrix4(pub [Vector4; 4]);

unsafe impl bytemuck::Zeroable for Matrix4 {}
unsafe impl bytemuck::Pod for Matrix4 {}

impl Matrix4 {
    #[inline]
    pub const fn new(x: Vector4, y: Vector4, z: Vector4, w: Vector4) -> Matrix4 {
        Matrix4([x, y, z, w])
    }

    #[inline]
    pub const fn identity() -> Matrix4 {
        Matrix4([
            Vector4([1.0, 0.0, 0.0, 0.0]),
            Vector4([0.0, 1.0, 0.0, 0.0]),
            Vector4([0.0, 0.0, 1.0, 0.0]),
            Vector4([0.0, 0.0, 0.0, 1.0]),
        ])
    }

    #[inline]
    pub fn translate(v: Vector3) -> Matrix4 {
        Matrix4([
            Vector4([1.0, 0.0, 0.0, 0.0]),
            Vector4([0.0, 1.0, 0.0, 0.0]),
            Vector4([0.0, 0.0, 1.0, 0.0]),
            Vector4([v.0[0], v.0[1], v.0[2], 1.0]),
        ])
    }

    #[inline]
    pub fn scale(v: Vector3) -> Matrix4 {
        Matrix4([
            Vector4([v.0[0], 0.0, 0.0, 0.0]),
            Vector4([0.0, v.0[1], 0.0, 0.0]),
            Vector4([0.0, 0.0, v.0[2], 0.0]),
            Vector4([0.0, 0.0, 0.0, 1.0]),
        ])
    }

    #[inline]
    pub fn rotate_right(angle: f32) -> Matrix4 {
        let sin_theta = angle.sin();
        let cos_theta = angle.cos();
        Matrix4([
            Vector4([1.0, 0.0, 0.0, 0.0]),
            Vector4([0.0, cos_theta, -sin_theta, 0.0]),
            Vector4([0.0, sin_theta, cos_theta, 0.0]),
            Vector4([0.0, 0.0, 0.0, 1.0]),
        ])
    }

    #[inline]
    pub fn rotate_up(angle: f32) -> Matrix4 {
        let sin_theta = angle.sin();
        let cos_theta = angle.cos();
        Matrix4([
            Vector4([cos_theta, 0.0, sin_theta, 0.0]),
            Vector4([0.0, 1.0, 0.0, 0.0]),
            Vector4([-sin_theta, 0.0, cos_theta, 0.0]),
            Vector4([0.0, 0.0, 0.0, 1.0]),
        ])
    }

    #[inline]
    pub fn rotate_forward(angle: f32) -> Matrix4 {
        let sin_theta = angle.sin();
        let cos_theta = angle.cos();
        Matrix4([
            Vector4([cos_theta, -sin_theta, 0.0, 0.0]),
            Vector4([sin_theta, cos_theta, 0.0, 0.0]),
            Vector4([0.0, 0.0, 1.0, 0.0]),
            Vector4([0.0, 0.0, 0.0, 1.0]),
        ])
    }

    #[inline]
    pub fn perspective(projection: &PerspectiveProjection) -> Matrix4 {
        let depth = projection.near - projection.far;
        let tan_fov = (projection.fov / 2.0).tan();
        Matrix4([
            Vector4([1.0 / (tan_fov * projection.aspect_ratio), 0.0, 0.0, 0.0]),
            Vector4([0.0, 1.0 / tan_fov, 0.0, 0.0]),
            Vector4([0.0, 0.0, (projection.near + projection.far) / depth, -1.0]),
            Vector4([
                0.0,
                0.0,
                (2.0 * projection.far * projection.near) / depth,
                0.0,
            ]),
        ])
    }

    #[inline]
    pub fn orthographic(
        top: f32,
        left: f32,
        bottom: f32,
        right: f32,
        near: f32,
        far: f32,
    ) -> Matrix4 {
        Matrix4([
            Vector4([2.0 / (right - left), 0.0, 0.0, 0.0]),
            Vector4([0.0, 2.0 / (top - bottom), 0.0, 0.0]),
            Vector4([0.0, 0.0, -2.0 / (far - near), 0.0]),
            Vector4([
                -((right + left) / (right - left)),
                -((top + bottom) / (top - bottom)),
                -((far + near) / (far - near)),
                1.0,
            ]),
        ])
    }

    #[inline]
    pub const fn vulkan_projection_correct() -> Matrix4 {
        Matrix4([
            Vector4([-1.0, 0.0, 0.0, 0.0]),
            Vector4([0.0, 1.0, 0.0, 0.0]),
            Vector4([0.0, 0.0, 0.5, 0.0]),
            Vector4([0.0, 0.0, 0.5, 1.0]),
        ])
    }

    #[inline]
    pub fn look_at(position: Vector3, at: Vector3, up: Vector3) -> Matrix4 {
        let z = (at - position).normalized();
        let x = z.cross(up).normalized();
        let y = x.cross(z);
        let z = -z;
        Matrix4([
            Vector4([x.0[0], y.0[0], z.0[0], 0.0]),
            Vector4([x.0[1], y.0[1], z.0[1], 0.0]),
            Vector4([x.0[2], y.0[2], z.0[2], 0.0]),
            Vector4([-x.dot(position), -y.dot(position), -z.dot(position), 1.0]),
        ])
    }

    #[rustfmt::skip]
    pub fn inversed(&self) -> Matrix4 {
        let a2323 = self.0[2].0[2] * self.0[3].0[3] - self.0[2].0[3] * self.0[3].0[2];
        let a1323 = self.0[2].0[1] * self.0[3].0[3] - self.0[2].0[3] * self.0[3].0[1];
        let a1223 = self.0[2].0[1] * self.0[3].0[2] - self.0[2].0[2] * self.0[3].0[1];
        let a0323 = self.0[2].0[0] * self.0[3].0[3] - self.0[2].0[3] * self.0[3].0[0];
        let a0223 = self.0[2].0[0] * self.0[3].0[2] - self.0[2].0[2] * self.0[3].0[0];
        let a0123 = self.0[2].0[0] * self.0[3].0[1] - self.0[2].0[1] * self.0[3].0[0];
        let a2313 = self.0[1].0[2] * self.0[3].0[3] - self.0[1].0[3] * self.0[3].0[2];
        let a1313 = self.0[1].0[1] * self.0[3].0[3] - self.0[1].0[3] * self.0[3].0[1];
        let a1213 = self.0[1].0[1] * self.0[3].0[2] - self.0[1].0[2] * self.0[3].0[1];
        let a2312 = self.0[1].0[2] * self.0[2].0[3] - self.0[1].0[3] * self.0[2].0[2];
        let a1312 = self.0[1].0[1] * self.0[2].0[3] - self.0[1].0[3] * self.0[2].0[1];
        let a1212 = self.0[1].0[1] * self.0[2].0[2] - self.0[1].0[2] * self.0[2].0[1];
        let a0313 = self.0[1].0[0] * self.0[3].0[3] - self.0[1].0[3] * self.0[3].0[0];
        let a0213 = self.0[1].0[0] * self.0[3].0[2] - self.0[1].0[2] * self.0[3].0[0];
        let a0312 = self.0[1].0[0] * self.0[2].0[3] - self.0[1].0[3] * self.0[2].0[0];
        let a0212 = self.0[1].0[0] * self.0[2].0[2] - self.0[1].0[2] * self.0[2].0[0];
        let a0113 = self.0[1].0[0] * self.0[3].0[1] - self.0[1].0[1] * self.0[3].0[0];
        let a0112 = self.0[1].0[0] * self.0[2].0[1] - self.0[1].0[1] * self.0[2].0[0];

        let det = 1.0 /
            (self.0[0].0[0] * ( self.0[1].0[1] * a2323 - self.0[1].0[2] * a1323 + self.0[1].0[3] * a1223)
            - self.0[0].0[1] * ( self.0[1].0[0] * a2323 - self.0[1].0[2] * a0323 + self.0[1].0[3] * a0223)
            + self.0[0].0[2] * ( self.0[1].0[0] * a1323 - self.0[1].0[1] * a0323 + self.0[1].0[3] * a0123)
            - self.0[0].0[3] * ( self.0[1].0[0] * a1223 - self.0[1].0[1] * a0223 + self.0[1].0[2] * a0123));

        Matrix4([
            Vector4([
                det * (self.0[1].0[1] * a2323 - self.0[1].0[2] * a1323 + self.0[1].0[3] * a1223),
                det * -(self.0[0].0[1] * a2323 - self.0[0].0[2] * a1323 + self.0[0].0[3] * a1223),
                det * (self.0[0].0[1] * a2313 - self.0[0].0[2] * a1313 + self.0[0].0[3] * a1213),
                det * -(self.0[0].0[1] * a2312 - self.0[0].0[2] * a1312 + self.0[0].0[3] * a1212),
            ]),

            Vector4([
                det * -(self.0[1].0[0] * a2323 - self.0[1].0[2] * a0323 + self.0[1].0[3] * a0223),
                det * (self.0[0].0[0] * a2323 - self.0[0].0[2] * a0323 + self.0[0].0[3] * a0223),
                det * -(self.0[0].0[0] * a2313 - self.0[0].0[2] * a0313 + self.0[0].0[3] * a0213),
                det * (self.0[0].0[0] * a2312 - self.0[0].0[2] * a0312 + self.0[0].0[3] * a0212),
            ]),

            Vector4([
                det * (self.0[1].0[0] * a1323 - self.0[1].0[1] * a0323 + self.0[1].0[3] * a0123),
                det * -(self.0[0].0[0] * a1323 - self.0[0].0[1] * a0323 + self.0[0].0[3] * a0123),
                det * (self.0[0].0[0] * a1313 - self.0[0].0[1] * a0313 + self.0[0].0[3] * a0113),
                det * -(self.0[0].0[0] * a1312 - self.0[0].0[1] * a0312 + self.0[0].0[3] * a0112),
            ]),

            Vector4([
                det * -(self.0[1].0[0] * a1223 - self.0[1].0[1] * a0223 + self.0[1].0[2] * a0123),
                det * (self.0[0].0[0] * a1223 - self.0[0].0[1] * a0223 + self.0[0].0[2] * a0123),
                det * -(self.0[0].0[0] * a1213 - self.0[0].0[1] * a0213 + self.0[0].0[2] * a0113),
                det * (self.0[0].0[0] * a1212 - self.0[0].0[1] * a0212 + self.0[0].0[2] * a0112),
            ])
        ])
    }

    #[inline]
    #[rustfmt::skip]
    pub fn transposed(&self) -> Matrix4 {
        Matrix4([
            Vector4([self.0[0].0[0], self.0[1].0[0], self.0[2].0[0], self.0[3].0[0]]),
            Vector4([self.0[0].0[1], self.0[1].0[1], self.0[2].0[1], self.0[3].0[1]]),
            Vector4([self.0[0].0[2], self.0[1].0[2], self.0[2].0[2], self.0[3].0[2]]),
            Vector4([self.0[0].0[3], self.0[1].0[3], self.0[2].0[3], self.0[3].0[3]]),
        ])
    }

    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl From<Quaternion> for Matrix4 {
    #[inline]
    #[rustfmt::skip]
    fn from(q: Quaternion) -> Matrix4 {
        Matrix4([
            Vector4([2.0 * (q.0[0] * q.0[2] - q.0[3] * q.0[1]), 2.0 * (q.0[1] * q.0[2] + q.0[3] * q.0[0]), 1.0 - 2.0 * (q.0[0] * q.0[0] + q.0[1] * q.0[1]), 0.0]),
            Vector4([1.0 - 2.0 * (q.0[1] * q.0[1] + q.0[2] * q.0[2]), 2.0 * (q.0[0] * q.0[1] - q.0[3] * q.0[2]), 2.0 * (q.0[0] * q.0[2] + q.0[3] * q.0[1]), 0.0]),
            Vector4([2.0 * (q.0[0] * q.0[1] + q.0[3] * q.0[2]), 1.0 - 2.0 * (q.0[0] * q.0[0] + q.0[2] * q.0[2]), 2.0 * (q.0[1] * q.0[2] - q.0[3] * q.0[0]), 0.0]),
            Vector4([0.0, 0.0, 0.0, 1.0]),
        ])
    }
}

impl Index<usize> for Matrix4 {
    type Output = Vector4;
    #[inline]
    fn index(&self, index: usize) -> &Vector4 {
        &self.0[index]
    }
}

impl IndexMut<usize> for Matrix4 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Vector4 {
        &mut self.0[index]
    }
}

impl Mul<&Matrix4> for &Matrix4 {
    type Output = Matrix4;

    // TODO: optimize / inline?
    fn mul(self, rhs: &Matrix4) -> Matrix4 {
        let mut ret = Matrix4::default();
        for i in 0..4 {
            for j in 0..4 {
                ret[i][0] += self[i][j] * rhs[j][0];
                ret[i][1] += self[i][j] * rhs[j][1];
                ret[i][2] += self[i][j] * rhs[j][2];
                ret[i][3] += self[i][j] * rhs[j][3];
            }
        }
        ret
    }
}
