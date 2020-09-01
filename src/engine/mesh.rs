use crate::{collections::PoolId, gfx::AnimatedMesh, math::Triangle3};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct SurfaceId(pub u64);

#[derive(Debug)]
pub struct Surface {
    triangle: Triangle3,
    flags: (),
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct CollisionMeshId(pub u64);

impl PoolId for CollisionMeshId {
    fn next(&self) -> CollisionMeshId {
        CollisionMeshId(self.0 + 1)
    }
}

#[derive(Debug)]
pub struct CollisionMesh {
    pub surfaces: Vec<Surface>,
}

impl From<AnimatedMesh> for CollisionMesh {
    fn from(mesh: AnimatedMesh) -> CollisionMesh {
        let surfaces = mesh
            .indices()
            .chunks_exact(3)
            .map(|indices| {
                [
                    mesh.vertices()[indices[0] as usize].position(),
                    mesh.vertices()[indices[1] as usize].position(),
                    mesh.vertices()[indices[2] as usize].position(),
                ]
            })
            .map(|vertices| Surface {
                triangle: vertices.into(),
                flags: (),
            })
            .collect();
        CollisionMesh { surfaces }
    }
}
