use crate::math::Vector3;
use sdl2::mixer::Chunk;

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Vertex {
    pub position: Vector3,
    pub normal: Vector3,
    pub diffuse: u32,
}

unsafe impl bytemuck::Zeroable for Vertex {}

unsafe impl bytemuck::Pod for Vertex {}

#[derive(Debug, Default)]
pub struct ChunkMesh {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

impl ChunkMesh {
    #[inline]
    pub fn vertices(&self) -> &Vec<Vertex> {
        &self.vertices
    }

    #[inline]
    pub fn indices(&self) -> &Vec<u32> {
        &self.indices
    }

    pub fn triangulate(&mut self, chunk: &Chunk) {}
}
