use smallvec::SmallVec;

use crate::{
    collections::pool::Handle,
    gfx::Transform,
    math::{Quaternion, Vector3},
};

#[derive(Debug)]
pub enum Renderer {}

// TODO: velocity verlet-integration.
//  I think it might be interesting to do scale as well for elastic things.
#[derive(Default, Debug)]
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
    renderer: Option<Renderer>,
    controller: Option<Handle<Controller>>,
}

#[derive(Default, Debug)]
pub struct Controller {
    handle: Handle<Controller>,
    children: SmallVec<[Handle<Entity>; 16]>,
    logic: SmallVec<[Logic; 16]>,
}

#[derive(Debug)]
pub enum Logic {}
