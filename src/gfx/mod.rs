use crate::math::Vector3;

mod frustum;
mod mesher;

pub use frustum::*;
pub use mesher::*;

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Vertex {
    pub position: Vector3,
    pub diffuse: Vector3,
}

unsafe impl bytemuck::Zeroable for Vertex {}

unsafe impl bytemuck::Pod for Vertex {}

#[derive(Debug, Default)]
pub struct Mesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl Mesh {
    #[inline]
    pub fn vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    #[inline]
    pub fn indices(&self) -> &Vec<u32> {
        &self.indices
    }
}

const CUBE_INDICES: [u32; 36] = [
    0, 1, 2, 2, 1, 3, 2, 3, 4, 4, 3, 5, 4, 5, 6, 6, 5, 7, 6, 7, 0, 0, 7, 1, 1, 7, 3, 3, 7, 5, 6, 0,
    4, 4, 0, 2,
];

const CUBE_VERTEX_POSITIONS: [Vector3; 8] = [
    Vector3::new(0.0, 0.0, 1.0),
    Vector3::new(1.0, 0.0, 1.0),
    Vector3::new(0.0, 1.0, 1.0),
    Vector3::new(1.0, 1.0, 1.0),
    Vector3::new(0.0, 1.0, 0.0),
    Vector3::new(1.0, 1.0, 0.0),
    Vector3::new(0.0, 0.0, 0.0),
    Vector3::new(1.0, 0.0, 0.0),
];
