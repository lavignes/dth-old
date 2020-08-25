mod matrix;
mod quaternion;
mod vector;

pub use matrix::*;
pub use quaternion::*;
pub use vector::*;

use std::f32;

const TAU: f32 = f32::consts::PI * 2.0;

const LOG2_TABLE_U64: [usize; 64] = [
    63, 0, 58, 1, 59, 47, 53, 2, 60, 39, 48, 27, 54, 33, 42, 3, 61, 51, 37, 40, 49, 18, 28, 20, 55,
    30, 34, 11, 43, 14, 22, 4, 62, 57, 46, 52, 38, 26, 32, 41, 50, 36, 17, 19, 29, 10, 13, 21, 56,
    45, 25, 31, 35, 16, 9, 12, 44, 24, 15, 8, 23, 7, 6, 5,
];

const LOG2_TABLE_U32: [usize; 32] = [
    0, 9, 1, 10, 13, 21, 2, 29, 11, 14, 16, 18, 22, 25, 3, 30, 8, 12, 20, 28, 15, 17, 24, 7, 19,
    27, 23, 6, 26, 5, 4, 31,
];

/// Wrap an angle in radians between \[0 - TAU\]
#[inline]
pub fn normalize_angle(angle: f32) -> f32 {
    if angle < 0.0 {
        angle + TAU
    } else if angle > TAU {
        TAU - angle
    } else {
        angle
    }
}

pub trait IntLog2 {
    fn log2(self) -> usize;
}

impl IntLog2 for u64 {
    fn log2(self) -> usize {
        const MAGIC: usize = 0x07ED_D5E5_9A4E_28C2;
        let mut value = self as usize;
        value |= value >> 1;
        value |= value >> 2;
        value |= value >> 4;
        value |= value >> 8;
        value |= value >> 16;
        value |= value >> 32;
        LOG2_TABLE_U64[((value - (value >> 1)) * MAGIC) >> 58]
    }
}

impl IntLog2 for u32 {
    fn log2(self) -> usize {
        const MAGIC: usize = 0x07C4_ACDD;
        let mut value = self as usize;
        value |= value >> 1;
        value |= value >> 2;
        value |= value >> 4;
        value |= value >> 8;
        value |= value >> 16;
        LOG2_TABLE_U32[(value * MAGIC) >> 27]
    }
}
