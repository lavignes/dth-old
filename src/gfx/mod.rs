use crate::{math::Vector3, world::Chunk};

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct Vertex {
    pub position: Vector3,
    pub diffuse: u32,
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
    Vector3::new(-0.5, -0.5, 0.5),
    Vector3::new(0.5, -0.5, 0.5),
    Vector3::new(-0.5, 0.5, 0.5),
    Vector3::new(0.5, 0.5, 0.5),
    Vector3::new(-0.5, 0.5, -0.5),
    Vector3::new(0.5, 0.5, -0.5),
    Vector3::new(-0.5, -0.5, -0.5),
    Vector3::new(0.5, -0.5, -0.5),
];

#[derive(Default)]
pub struct ChunkMesher {}

impl ChunkMesher {
    pub fn mesh(&self, chunk: &Chunk, mesh: &mut Mesh) {
        mesh.indices.clear();
        mesh.vertices.clear();
        let mut index_offset = 0;
        // Get the world-space location of the chunk to build the vertex list
        let origin = Vector3::new(chunk.position().x(), 0.0, chunk.position().y());
        for section in chunk.sections() {
            for (index, _tile) in section.cube().iter_indexed() {
                let index_parts: (usize, usize, usize) = index.into();
                let offset = origin + Vector3::from(index_parts);
                mesh.vertices
                    .extend(CUBE_VERTEX_POSITIONS.iter().map(|pos| Vertex {
                        position: *pos + offset,
                        diffuse: offset.y() as u32,
                    }));
                index_offset += CUBE_VERTEX_POSITIONS.len();
                mesh.indices.extend(
                    CUBE_INDICES
                        .iter()
                        .map(|index| *index + index_offset as u32),
                );
            }
        }

        log::debug!(
            "Meshed a chunk -- {} vertices {} indices",
            mesh.vertices.len(),
            mesh.indices.len()
        );
    }
}
