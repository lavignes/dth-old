use crate::math::Vector3;

#[derive(Debug, Default)]
pub struct AnimatedVertex {
    position: Vector3,
    normal: Vector3,
}

impl AnimatedVertex {
    #[inline]
    pub fn new(position: Vector3, normal: Vector3) -> AnimatedVertex {
        AnimatedVertex { position, normal }
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
