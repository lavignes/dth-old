use crate::collections::PoolObject;
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
    transform: Transform,
    collider: Option<Collider>,
    render_mode: Option<RenderMode>,
    prefab: Option<Prefab>,
}

impl Actor {
    #[inline]
    pub fn transform(&self) -> &Transform {
        &self.transform
    }

    #[inline]
    pub fn transform_mut(&mut self) -> &mut Transform {
        &mut self.transform
    }

    #[inline]
    pub fn render_mode(&self) -> Option<&RenderMode> {
        self.render_mode.as_ref()
    }
}

impl PoolObject for Actor {
    #[inline]
    fn clear(&mut self) {
        // TODO: make this more efficient.
        self.transform.clear();
        self.collider = None;
        self.render_mode = None;
        if let Some(prefab) = &mut self.prefab {
            Prefab::clear(prefab);
        }
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
