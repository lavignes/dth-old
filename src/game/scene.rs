use crate::{
    collections::Pool,
    game::{Camera, Entity},
};

pub struct Scene {
    camera: Camera,
    entities: Pool<Entity>,
}
