use crate::collections::PoolObject;
use crate::{
    collections::PoolId,
    engine::{CollisionMeshId, SurfaceId},
    gfx::NodeId,
};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct GeometryId(pub u64);

impl PoolId for GeometryId {
    fn next(&self) -> GeometryId {
        GeometryId(self.0 + 1)
    }
}

/// World geometry. Things that don't *necessarily* move or "think".
/// Think floors, walls, and ceilings.
#[derive(Debug)]
pub enum Geometry {
    StaticMap(StaticMap),
    Todo,
}

impl Default for Geometry {
    #[inline]
    fn default() -> Geometry {
        Geometry::Todo
    }
}

impl PoolObject for Geometry {
    #[inline]
    fn clear(&mut self) {
        match self {
            Geometry::StaticMap(map) => map.clear(),
            _ => todo!("{:?}", self),
        }
    }
}

#[derive(Debug)]
pub struct Sector {
    surfaces: Vec<SurfaceId>,
}

#[derive(Debug, Default)]
pub struct StaticMap {
    collision_mesh: Option<CollisionMeshId>,
    sectors: Vec<Sector>,
    render_node: NodeId,
}

impl StaticMap {
    #[inline]
    pub fn clear(&mut self) {
        self.collision_mesh = None;
        self.sectors.clear();
    }

    #[inline]
    pub fn render_node(&self) -> NodeId {
        self.render_node
    }
}
