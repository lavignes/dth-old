mod bitmap;
mod collada;
mod frustum;
mod mesh;

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
