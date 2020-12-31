use crate::math::{Vector2, Vector3, Vector4};

// TODO: Animated mesh?
// #[derive(Debug, Default)]
// pub struct AnimatedMaterialMesh {
//     inner: StaticMaterialMesh,
//     bone_indices: Vec<(u8, u8)>,
//     bone_weights: Vec<Vector2>,
// }

#[derive(Debug, Default)]
pub struct StaticMaterialMesh {
    vertices: Vec<StaticMaterialVertex>,
    indices: Vec<u32>,
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct StaticMaterialVertex {
    position: Vector3,
    normal: Vector3,
    tex_coord: Vector2,
    color: Vector4,
}

impl StaticMaterialVertex {
    #[inline]
    pub fn new(
        position: Vector3,
        normal: Vector3,
        tex_coord: Vector2,
        color: Vector4,
    ) -> StaticMaterialVertex {
        StaticMaterialVertex {
            position,
            normal,
            tex_coord,
            color,
        }
    }
}

unsafe impl bytemuck::Zeroable for StaticMaterialVertex {}

unsafe impl bytemuck::Pod for StaticMaterialVertex {}

impl StaticMaterialMesh {
    #[inline]
    pub fn clear(&mut self) {
        self.vertices.clear();
        self.indices.clear();
    }

    #[inline]
    pub fn add_vertex(&mut self, vertex: StaticMaterialVertex) {
        self.vertices.push(vertex);
    }

    #[inline]
    pub fn vertices(&self) -> &[StaticMaterialVertex] {
        &self.vertices
    }

    #[inline]
    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    #[inline]
    pub fn add_index(&mut self, index: u32) {
        self.indices.push(index);
    }
}
