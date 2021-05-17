use crate::gfx::{OrthographicProjection, PerspectiveProjection, Transform};

pub enum Projection {
    Perspective(PerspectiveProjection),
    Orthographic(OrthographicProjection),
}

pub struct Camera {
    transform: Transform,
    projection: Projection,
}
