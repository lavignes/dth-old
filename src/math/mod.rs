mod matrix;
mod quaternion;
mod triangle;
mod vector;

pub use matrix::*;
pub use quaternion::*;
pub use triangle::*;
pub use vector::*;

use std::f32;

/// Wrap an angle in radians between \[0 - TAU\]
#[inline]
pub fn normalize_angle(angle: f32) -> f32 {
    let angle_mod = angle % f32::consts::TAU;
    if angle_mod < 0.0 {
        angle_mod + f32::consts::TAU
    } else {
        angle_mod
    }
}

#[inline]
pub fn clamp<T>(value: T, min: T, max: T) -> T
where
    T: PartialOrd,
{
    let mut clamp = value;
    if clamp < min {
        clamp = min;
    }
    if clamp > max {
        clamp = max;
    }
    clamp
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Float16(u16);

unsafe impl bytemuck::Zeroable for Float16 {}

unsafe impl bytemuck::Pod for Float16 {}

impl From<f32> for Float16 {
    #[inline]
    fn from(f: f32) -> Float16 {
        let x: u32 = bytemuck::cast(f);
        Float16(
            (((x >> 16) & 0x8000)
                | ((((x & 0x7f800000) - 0x38000000) >> 13) & 0x7c00)
                | ((x >> 13) & 0x03ff)) as u16,
        )
    }
}

impl Into<f32> for Float16 {
    #[inline]
    fn into(self) -> f32 {
        let x = self.0 as u32;
        bytemuck::cast((x & 0x8000) << 16 | (((x & 0x7c00) + 0x1C000) << 13) | ((x & 0x03FF) << 13))
    }
}
