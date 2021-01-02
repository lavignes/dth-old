use smallvec::SmallVec;

use crate::{
    collections::pool::Handle,
    gfx::Transform,
    math::{Quaternion, Vector3},
};

#[derive(Debug)]
pub enum EntityRenderer {}

// TODO: velocity verlet-integration.
//  I think it might be interesting to do scale as well for elastic things.
#[derive(Debug)]
pub struct Motion {
    velocity: Vector3,
    angular_velocity: Quaternion,
}

// Fat *sparse* entity system. It is pretty ECS-like but every entity has every component.
// The are controlled by generally 1 controller, but there is no theoretical limit.
// This idea is based on the entity system in Handmade Hero.
#[derive(Default, Debug)]
pub struct Entity {
    handle: Handle<Entity>,
    transform: Transform,
    // TODO: should it be option? *probably* since we can branch over a lot of logic.
    movement: Option<Motion>,
    renderer: Option<EntityRenderer>,
    // TODO: Probably dont need this since controllers can control multiple entities.
    //   I think this was for creating small scene graphs. But that probably is crazy.
    //   In fact, removing the scene graph simplifies the transform. Everything can be in
    //   world-space!
    // children: ...,
    controller: Option<Handle<Controller>>,
}

#[derive(Debug)]
pub struct Controller {
    children: SmallVec<[Handle<Entity>; 16]>,
    logic: Logic,
}

// TODO: traits and dynamic dispatch honestly start to make sense here :/
//  Though I suppose you can start doing **AND** relationships of logic...
#[derive(Debug)]
pub enum Logic {}
