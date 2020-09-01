use crate::{
    collections::PoolId,
    math::{Vector2, Vector3},
};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct RenderMeshId(pub u64);

impl PoolId for RenderMeshId {
    #[inline]
    fn next(&self) -> RenderMeshId {
        RenderMeshId(self.0 + 1)
    }
}

#[derive(Debug)]
pub enum RenderMesh {
    AnimatedMesh(AnimatedMesh),
    Todo,
}

impl Default for RenderMesh {
    #[inline]
    fn default() -> RenderMesh {
        RenderMesh::Todo
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct AnimatedVertex {
    position: Vector3,
    normal: Vector3,
    tex_coords: Vector2,
    bone_indices: Vector2,
    bone_weights: Vector2,
}

unsafe impl bytemuck::Zeroable for AnimatedVertex {}

unsafe impl bytemuck::Pod for AnimatedVertex {}

impl AnimatedVertex {
    #[inline]
    pub fn new(position: Vector3, normal: Vector3) -> AnimatedVertex {
        AnimatedVertex {
            position,
            normal,
            ..AnimatedVertex::default()
        }
    }

    #[inline]
    pub fn position(&self) -> Vector3 {
        self.position
    }

    #[inline]
    pub fn normal(&self) -> Vector3 {
        self.normal
    }
}

#[derive(Debug, Default)]
pub struct AnimatedMesh {
    vertices: Vec<AnimatedVertex>,
    indices: Vec<u32>,
}

impl AnimatedMesh {
    #[inline]
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    #[inline]
    pub fn vertices(&self) -> &[AnimatedVertex] {
        &self.vertices
    }

    #[inline]
    pub fn vertices_mut(&mut self) -> &mut [AnimatedVertex] {
        &mut self.vertices
    }

    #[inline]
    pub fn add_vertex(&mut self, vertex: AnimatedVertex) {
        self.vertices.push(vertex);
    }

    #[inline]
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    #[inline]
    pub fn indices_mut(&mut self) -> &mut [u32] {
        &mut self.indices
    }

    #[inline]
    pub fn add_index(&mut self, index: u32) {
        self.indices.push(index);
    }
}
