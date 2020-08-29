mod matrix;
mod quaternion;
mod vector;

pub use matrix::*;
pub use quaternion::*;
pub use vector::*;

use std::f32;

pub const TAU: f32 = f32::consts::PI * 2.0;

/// Wrap an angle in radians between \[0 - TAU\]
#[inline]
pub fn normalize_angle(angle: f32) -> f32 {
    let angle_mod = angle % TAU;
    if angle_mod < 0.0 {
        angle_mod + TAU
    } else {
        angle_mod
    }
}

pub fn clamp_angle(angle: f32, min: f32, max: f32) -> f32 {
    let mut clamp = angle;
    if clamp < min {
        clamp = min;
    }
    if clamp > max {
        clamp = max;
    }
    clamp
}
