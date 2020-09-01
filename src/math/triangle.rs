use crate::math::Vector3;

#[derive(Default, Debug)]
pub struct Triangle3 {
    pub vertices: [Vector3; 3],
    pub normal: Vector3,
}

impl Triangle3 {
    #[inline]
    pub fn new(vertices: [Vector3; 3]) -> Triangle3 {
        Triangle3 {
            vertices,
            normal: (vertices[0] - vertices[1])
                .cross(vertices[0] - vertices[2])
                .normalized(),
        }
    }
}

impl From<[Vector3; 3]> for Triangle3 {
    #[inline]
    fn from(vertices: [Vector3; 3]) -> Triangle3 {
        Triangle3::new(vertices)
    }
}
