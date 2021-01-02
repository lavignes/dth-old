use crate::math::Quaternion;
use std::{
    cmp::PartialEq,
    convert::From,
    f32,
    ops::{Add, AddAssign, Div, Index, IndexMut, Mul, MulAssign, Neg, Sub, SubAssign},
};

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Vector2(pub [f32; 2]);

unsafe impl bytemuck::Zeroable for Vector2 {}

unsafe impl bytemuck::Pod for Vector2 {}

impl Vector2 {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Vector2 {
        Vector2([x, y])
    }

    #[inline]
    pub fn x(&self) -> f32 {
        self.0[0]
    }

    #[inline]
    pub fn set_x(&mut self, x: f32) {
        self.0[0] = x
    }

    #[inline]
    pub fn y(&self) -> f32 {
        self.0[1]
    }

    #[inline]
    pub fn set_y(&mut self, y: f32) {
        self.0[1] = y
    }

    #[inline]
    pub fn widened(&self, z: f32) -> Vector3 {
        Vector3([self.0[0], self.0[1], z])
    }

    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl PartialEq for Vector2 {
    #[inline]
    fn eq(&self, rhs: &Vector2) -> bool {
        (self.0[0] - rhs.0[0]).abs() <= f32::EPSILON && (self.0[1] - rhs.0[1]).abs() <= f32::EPSILON
    }
}

impl Add for Vector2 {
    type Output = Vector2;
    #[inline]
    fn add(self, rhs: Vector2) -> Vector2 {
        Vector2([self.0[0] + rhs.0[0], self.0[1] + rhs.0[1]])
    }
}

impl AddAssign for Vector2 {
    #[inline]
    fn add_assign(&mut self, rhs: Vector2) {
        self.0[0] += rhs.0[0];
        self.0[1] += rhs.0[1];
    }
}

impl Sub for Vector2 {
    type Output = Vector2;
    #[inline]
    fn sub(self, rhs: Vector2) -> Vector2 {
        Vector2([self.0[0] - rhs.0[0], self.0[1] - rhs.0[1]])
    }
}

impl MulAssign for Vector2 {
    #[inline]
    fn mul_assign(&mut self, rhs: Vector2) {
        self.0[0] *= rhs.0[0];
        self.0[1] *= rhs.0[1];
    }
}

impl Div for Vector2 {
    type Output = Vector2;
    #[inline]
    fn div(self, rhs: Vector2) -> Vector2 {
        Vector2([self.0[0] / rhs.0[0], self.0[1] / rhs.0[1]])
    }
}

impl Mul<f32> for Vector2 {
    type Output = Vector2;
    #[inline]
    fn mul(self, rhs: f32) -> Vector2 {
        Vector2([self.0[0] * rhs, self.0[1] * rhs])
    }
}

impl Div<f32> for Vector2 {
    type Output = Vector2;
    #[inline]
    fn div(self, rhs: f32) -> Vector2 {
        Vector2([self.0[0] / rhs, self.0[1] / rhs])
    }
}

impl Neg for Vector2 {
    type Output = Vector2;
    #[inline]
    fn neg(self) -> Vector2 {
        Vector2([-self.0[0], -self.0[1]])
    }
}

impl Index<usize> for Vector2 {
    type Output = f32;
    #[inline]
    fn index(&self, index: usize) -> &f32 {
        &self.0[index]
    }
}

impl IndexMut<usize> for Vector2 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        &mut self.0[index]
    }
}

impl AsRef<[f32]> for Vector2 {
    #[inline]
    fn as_ref(&self) -> &[f32] {
        &self.0
    }
}

impl From<(f32, f32)> for Vector2 {
    #[inline]
    fn from(value: (f32, f32)) -> Vector2 {
        Vector2([value.0, value.1])
    }
}

impl From<(i32, i32)> for Vector2 {
    #[inline]
    fn from(value: (i32, i32)) -> Vector2 {
        Vector2([value.0 as f32, value.1 as f32])
    }
}

impl From<(u32, u32)> for Vector2 {
    #[inline]
    fn from(value: (u32, u32)) -> Vector2 {
        Vector2([value.0 as f32, value.1 as f32])
    }
}

impl Into<(u32, u32)> for Vector2 {
    #[inline]
    fn into(self: Vector2) -> (u32, u32) {
        (self.0[0] as u32, self.0[1] as u32)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Vector3(pub [f32; 3]);

unsafe impl bytemuck::Zeroable for Vector3 {}

unsafe impl bytemuck::Pod for Vector3 {}

impl Vector3 {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Vector3 {
        Vector3([x, y, z])
    }

    #[inline]
    pub const fn splat(f: f32) -> Vector3 {
        Vector3([f, f, f])
    }

    #[inline]
    pub fn x(&self) -> f32 {
        self.0[0]
    }

    #[inline]
    pub fn set_x(&mut self, x: f32) {
        self.0[0] = x
    }

    #[inline]
    pub fn y(&self) -> f32 {
        self.0[1]
    }

    #[inline]
    pub fn set_y(&mut self, y: f32) {
        self.0[1] = y
    }

    #[inline]
    pub fn z(&self) -> f32 {
        self.0[2]
    }

    #[inline]
    pub fn set_z(&mut self, z: f32) {
        self.0[2] = z
    }

    #[inline]
    pub const fn up() -> Vector3 {
        Vector3([0.0, 1.0, 0.0])
    }

    #[inline]
    pub const fn down() -> Vector3 {
        Vector3([0.0, 1.0, 0.0])
    }

    #[inline]
    pub const fn right() -> Vector3 {
        Vector3([1.0, 0.0, 0.0])
    }

    #[inline]
    pub const fn left() -> Vector3 {
        Vector3([-1.0, 0.0, 0.0])
    }

    #[inline]
    pub const fn forward() -> Vector3 {
        Vector3([0.0, 0.0, 1.0])
    }

    #[inline]
    pub const fn backward() -> Vector3 {
        Vector3([0.0, 0.0, -1.0])
    }

    #[inline]
    pub fn sin(&self) -> Vector3 {
        Vector3([self.0[0].sin(), self.0[1].sin(), self.0[2].sin()])
    }

    #[inline]
    pub fn cos(&self) -> Vector3 {
        Vector3([self.0[0].cos(), self.0[1].cos(), self.0[2].cos()])
    }

    #[inline]
    pub fn widened(&self, w: f32) -> Vector4 {
        Vector4([self.0[0], self.0[1], self.0[2], w])
    }

    #[inline]
    pub fn length(&self) -> f32 {
        self.squared_normal().sqrt()
    }

    #[inline]
    pub fn squared_normal(&self) -> f32 {
        self.dot(*self)
    }

    #[inline]
    pub fn normalized(&self) -> Vector3 {
        self / self.length()
    }

    #[inline]
    pub fn cross(&self, rhs: Vector3) -> Vector3 {
        Vector3([
            self.0[1] * rhs.0[2] - self.0[2] * rhs.0[1],
            self.0[2] * rhs.0[0] - self.0[0] * rhs.0[2],
            self.0[0] * rhs.0[1] - self.0[1] * rhs.0[0],
        ])
    }

    #[inline]
    pub fn rotated(&self, rotation: Quaternion) -> Vector3 {
        (rotation * *self * rotation.conjugated()).0.narrowed()
    }

    #[inline]
    pub fn dot(&self, rhs: Vector3) -> f32 {
        (self.0[0] * rhs.0[0]) + (self.0[1] * rhs.0[1]) + (self.0[2] * rhs.0[2])
    }

    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl PartialEq for Vector3 {
    #[inline]
    fn eq(&self, rhs: &Vector3) -> bool {
        (self.0[0] - rhs.0[0]).abs() <= f32::EPSILON
            && (self.0[1] - rhs.0[1]).abs() <= f32::EPSILON
            && (self.0[2] - rhs.0[2]).abs() <= f32::EPSILON
    }
}

impl AddAssign for Vector3 {
    #[inline]
    fn add_assign(&mut self, rhs: Vector3) {
        self.0[0] += rhs.0[0];
        self.0[1] += rhs.0[1];
        self.0[2] += rhs.0[2];
    }
}

impl SubAssign for Vector3 {
    #[inline]
    fn sub_assign(&mut self, rhs: Vector3) {
        self.0[0] -= rhs.0[0];
        self.0[1] -= rhs.0[1];
        self.0[2] -= rhs.0[2];
    }
}

impl Add for Vector3 {
    type Output = Vector3;
    #[inline]
    fn add(self, rhs: Vector3) -> Vector3 {
        Vector3([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
        ])
    }
}

impl Sub for Vector3 {
    type Output = Vector3;
    #[inline]
    fn sub(self, rhs: Vector3) -> Vector3 {
        Vector3([
            self.0[0] - rhs.0[0],
            self.0[1] - rhs.0[1],
            self.0[2] - rhs.0[2],
        ])
    }
}

impl Mul for Vector3 {
    type Output = Vector3;
    #[inline]
    fn mul(self, rhs: Vector3) -> Vector3 {
        Vector3([
            self.0[0] * rhs.0[0],
            self.0[1] * rhs.0[1],
            self.0[2] * rhs.0[2],
        ])
    }
}

impl Neg for Vector3 {
    type Output = Vector3;
    #[inline]
    fn neg(self) -> Vector3 {
        Vector3([-self.0[0], -self.0[1], -self.0[2]])
    }
}

impl Add<f32> for Vector3 {
    type Output = Vector3;
    #[inline]
    fn add(self, rhs: f32) -> Vector3 {
        Vector3([self.0[0] + rhs, self.0[1] + rhs, self.0[2] + rhs])
    }
}

impl Sub<f32> for Vector3 {
    type Output = Vector3;
    #[inline]
    fn sub(self, rhs: f32) -> Vector3 {
        Vector3([self.0[0] - rhs, self.0[1] - rhs, self.0[2] - rhs])
    }
}

impl Mul<f32> for Vector3 {
    type Output = Vector3;
    #[inline]
    fn mul(self, rhs: f32) -> Vector3 {
        Vector3([self.0[0] * rhs, self.0[1] * rhs, self.0[2] * rhs])
    }
}

impl Div<f32> for Vector3 {
    type Output = Vector3;
    #[inline]
    fn div(self, rhs: f32) -> Vector3 {
        Vector3([self.0[0] / rhs, self.0[1] / rhs, self.0[2] / rhs])
    }
}

impl Div<f32> for &Vector3 {
    type Output = Vector3;
    #[inline]
    fn div(self, rhs: f32) -> Vector3 {
        Vector3([self.0[0] / rhs, self.0[1] / rhs, self.0[2] / rhs])
    }
}

impl Index<usize> for Vector3 {
    type Output = f32;
    #[inline]
    fn index(&self, index: usize) -> &f32 {
        &self.0[index]
    }
}

impl IndexMut<usize> for Vector3 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        &mut self.0[index]
    }
}

impl AsRef<[f32]> for Vector3 {
    #[inline]
    fn as_ref(&self) -> &[f32] {
        &self.0
    }
}

impl From<(f32, f32, f32)> for Vector3 {
    #[inline]
    fn from(value: (f32, f32, f32)) -> Vector3 {
        Vector3([value.0, value.1, value.2])
    }
}

impl From<(usize, usize, usize)> for Vector3 {
    #[inline]
    fn from(value: (usize, usize, usize)) -> Vector3 {
        Vector3([value.0 as f32, value.1 as f32, value.2 as f32])
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Vector4(pub [f32; 4]);

unsafe impl bytemuck::Zeroable for Vector4 {}

unsafe impl bytemuck::Pod for Vector4 {}

impl Vector4 {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Vector4 {
        Vector4([x, y, z, w])
    }

    #[inline]
    pub const fn splat(f: f32) -> Vector4 {
        Vector4([f, f, f, f])
    }

    #[inline]
    pub fn x(&self) -> f32 {
        self.0[0]
    }

    #[inline]
    pub fn set_x(&mut self, x: f32) {
        self.0[0] = x
    }

    #[inline]
    pub fn y(&self) -> f32 {
        self.0[1]
    }

    #[inline]
    pub fn set_y(&mut self, y: f32) {
        self.0[1] = y
    }

    #[inline]
    pub fn z(&self) -> f32 {
        self.0[2]
    }

    #[inline]
    pub fn set_z(&mut self, z: f32) {
        self.0[2] = z
    }

    #[inline]
    pub fn w(&self) -> f32 {
        self.0[3]
    }

    #[inline]
    pub fn set_w(&mut self, w: f32) {
        self.0[3] = w
    }

    #[inline]
    pub fn length(&self) -> f32 {
        self.squared_normal().sqrt()
    }

    #[inline]
    pub fn squared_normal(&self) -> f32 {
        self.dot(*self)
    }

    #[inline]
    pub fn normalized(&self) -> Vector4 {
        self / self.length()
    }

    #[inline]
    pub fn narrowed(&self) -> Vector3 {
        Vector3([self.0[0], self.0[1], self.0[2]])
    }

    #[inline]
    pub fn dot(&self, rhs: Vector4) -> f32 {
        (self.0[0] * rhs.0[0])
            + (self.0[1] * rhs.0[1])
            + (self.0[2] * rhs.0[2])
            + (self.0[3] * rhs.0[3])
    }

    #[inline]
    pub fn to_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }
}

impl PartialEq for Vector4 {
    #[inline]
    fn eq(&self, rhs: &Vector4) -> bool {
        (self.0[0] - rhs.0[0]).abs() <= f32::EPSILON
            && (self.0[1] - rhs.0[1]).abs() <= f32::EPSILON
            && (self.0[2] - rhs.0[2]).abs() <= f32::EPSILON
            && (self.0[3] - rhs.0[3]).abs() <= f32::EPSILON
    }
}

impl Neg for Vector4 {
    type Output = Vector4;
    #[inline]
    fn neg(self) -> Vector4 {
        Vector4([-self.0[0], -self.0[1], -self.0[2], -self.0[3]])
    }
}

impl Index<usize> for Vector4 {
    type Output = f32;
    #[inline]
    fn index(&self, index: usize) -> &f32 {
        &self.0[index]
    }
}

impl IndexMut<usize> for Vector4 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut f32 {
        &mut self.0[index]
    }
}

impl Add for Vector4 {
    type Output = Vector4;
    #[inline]
    fn add(self, rhs: Vector4) -> Vector4 {
        Vector4([
            self.0[0] + rhs.0[0],
            self.0[1] + rhs.0[1],
            self.0[2] + rhs.0[2],
            self.0[3] + rhs.0[3],
        ])
    }
}

impl Sub for Vector4 {
    type Output = Vector4;
    #[inline]
    fn sub(self, rhs: Vector4) -> Vector4 {
        Vector4([
            self.0[0] - rhs.0[0],
            self.0[1] - rhs.0[1],
            self.0[2] - rhs.0[2],
            self.0[3] - rhs.0[3],
        ])
    }
}

impl Div<f32> for Vector4 {
    type Output = Vector4;
    #[inline]
    fn div(self, rhs: f32) -> Vector4 {
        Vector4([
            self.0[0] / rhs,
            self.0[1] / rhs,
            self.0[2] / rhs,
            self.0[3] / rhs,
        ])
    }
}

impl Div<f32> for &Vector4 {
    type Output = Vector4;
    #[inline]
    fn div(self, rhs: f32) -> Vector4 {
        Vector4([
            self.0[0] / rhs,
            self.0[1] / rhs,
            self.0[2] / rhs,
            self.0[3] / rhs,
        ])
    }
}

impl Mul<f32> for Vector4 {
    type Output = Vector4;
    #[inline]
    fn mul(self, rhs: f32) -> Vector4 {
        Vector4([
            self.0[0] * rhs,
            self.0[1] * rhs,
            self.0[2] * rhs,
            self.0[3] * rhs,
        ])
    }
}

impl AsRef<[f32]> for Vector4 {
    #[inline]
    fn as_ref(&self) -> &[f32] {
        &self.0
    }
}

impl From<(f32, f32, f32, f32)> for Vector4 {
    #[inline]
    fn from(value: (f32, f32, f32, f32)) -> Vector4 {
        Vector4([value.0, value.1, value.2, value.3])
    }
}
