use crate::{
    collections::PoolId,
    engine::CollisionMesh,
    engine::Geometry,
    engine::GeometryId,
    game::Prefab,
    gfx::{NodeId, Transform},
};
use std::fmt::Debug;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct ActorId(pub u64);

impl PoolId for ActorId {
    fn next(&self) -> ActorId {
        ActorId(self.0 + 1)
    }
}

pub enum Collision<'a> {
    Actor(ActorId, &'a Actor),
    Geometry(GeometryId, &'a Geometry),
}

#[derive(Default, Debug)]
pub struct Actor {
    pub transform: Transform,
    pub collider: Option<Collider>,
    pub render_mode: Option<RenderMode>,
    pub prefab: Option<Prefab>,
}

impl Actor {
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }
}

#[derive(Debug)]
pub enum RenderMode {
    Node(NodeId),
}

#[derive(Debug)]
pub enum Collider {
    Cylinder { radius: f32, height: f32 },
    Mesh(CollisionMesh),
}
