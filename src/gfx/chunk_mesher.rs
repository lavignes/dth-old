use crate::{
    math::Vector3,
    tile::{TileFace, TileState},
    world::Chunk,
};
use std::time::Instant;

#[derive(Default, Debug)]
pub struct ChunkMesher {
    tile_mask: Vec<Option<TileState>>,
}

impl ChunkMesher {
    pub fn greedy(&mut self, chunk: &Chunk, mesh: &mut ChunkMesh) {
        let start = Instant::now();

        mesh.vertices.clear();
        mesh.indices.clear();
        let mut index_offset = 0;

        // TODO: 2d palette vec?
        self.tile_mask.clear();
        self.tile_mask.extend((0..(16 * 16)).map(|_| None));

        // TODO: I think when we make the mask, we can detect whether our face is not worth
        //  rendering by looking at whether the next face is transparent or not.

        for (cube_index, cube) in chunk
            .sections()
            .iter()
            .enumerate()
            .filter(|(_, section)| !section.is_empty())
            .map(|(index, section)| (index, section.cube().unwrap()))
        {
            // Test back-faces then front-faces
            for &backface in &[true, false] {
                // Sweep over all three dimensions
                for d in 0..3 {
                    let u = (d + 1) % 3;
                    let v = (d + 2) % 3;

                    // The tiles we're adding faces to.
                    let mut cursor = Vector3::default();

                    // The vector in the direction we are testing. To look at the "next" tile.
                    let mut normal = Vector3::default();
                    normal[d] = 1.0;

                    // The face of the "next" tile.
                    let _face = match (d, backface) {
                        (0, true) => TileFace::Left,
                        (0, false) => TileFace::Right,
                        (1, true) => TileFace::Bottom,
                        (1, false) => TileFace::Top,
                        (2, true) => TileFace::Back,
                        (2, false) => TileFace::Front,
                        _ => unreachable!(),
                    };

                    cursor[d] = -1.0;
                    while cursor[d] < 16.0 {
                        let mut mask_index = 0;
                        cursor[v] = 0.0;
                        while cursor[v] < 16.0 {
                            cursor[u] = 0.0;
                            while cursor[u] < 16.0 {
                                let tile_front = if cursor[d] >= 0.0 {
                                    Some(cube.get(cursor.into()))
                                } else {
                                    None
                                };

                                let tile_back = if cursor[d] < 16.0 - 1.0 {
                                    Some(cube.get((cursor + normal).into()))
                                } else {
                                    None
                                };

                                // TODO: simplify? ( the some check is probably not required )
                                if tile_front.is_some()
                                    && tile_back.is_some()
                                    && tile_front == tile_back
                                {
                                    self.tile_mask[mask_index] = None;
                                } else {
                                    self.tile_mask[mask_index] = if backface {
                                        tile_back.copied()
                                    } else {
                                        tile_front.copied()
                                    };
                                }
                                mask_index += 1;
                                cursor[u] += 1.0;
                            }
                            cursor[v] += 1.0;
                        }
                        cursor[d] += 1.0;

                        // TODO: dirty bit. If we never write to the mask above, then we can skip all of this loop
                        let mut mask_index = 0;
                        for j in 0..16 {
                            let mut i = 0;
                            while i < 16 {
                                // TODO: Do we need a null tile separate from id 0?
                                if let Some(tile) = self.tile_mask[mask_index] {
                                    // Find the width
                                    let mut width = 1;
                                    while i + width < 16
                                        && self.tile_mask[mask_index + width] == Some(tile)
                                    {
                                        width += 1;
                                    }

                                    // Find the height
                                    let mut height = 1;
                                    'outer: while j + height < 16 {
                                        for k in 0..width {
                                            let next_tile =
                                                self.tile_mask[mask_index + k + height * 16];
                                            if next_tile != Some(tile) {
                                                break 'outer;
                                            }
                                        }
                                        height += 1;
                                    }

                                    // TODO: check that the face is not transparent
                                    if !tile.is_void() {
                                        cursor[u] = i as f32;
                                        cursor[v] = j as f32;

                                        // Add in the offset of the cube
                                        let chunk_offset = chunk.position()
                                            + Vector3::new(0.0, 16.0 * cube_index as f32, 0.0);

                                        let mut scale_width = Vector3::default();
                                        scale_width.0[u] = width as f32;

                                        let mut scale_height = Vector3::default();
                                        scale_height.0[v] = height as f32;

                                        mesh.vertices.extend(
                                            [
                                                cursor,
                                                cursor + scale_height,
                                                cursor + scale_width,
                                                cursor + scale_width + scale_height,
                                            ]
                                            .iter()
                                            .map(
                                                |&position| ChunkVertex {
                                                    position: position + chunk_offset,
                                                    diffuse: match tile.id().0 {
                                                        1 => (0.0, 0.0, 1.0).into(),
                                                        2 => (1.0, 1.0, 0.0).into(),
                                                        _ => (1.0, 0.0, 0.0).into(),
                                                    },
                                                },
                                            ),
                                        );

                                        // TODO make new constants for the indices
                                        if backface {
                                            mesh.indices.extend(
                                                [2, 0, 1, 1, 3, 2]
                                                    .iter()
                                                    .map(|index| index + index_offset),
                                            );
                                        } else {
                                            mesh.indices.extend(
                                                [2, 3, 1, 1, 0, 2]
                                                    .iter()
                                                    .map(|index| index + index_offset),
                                            );
                                        }
                                        // Remember we are offsetting by the number of verts (4)
                                        index_offset += 4;
                                    }

                                    for l in 0..height {
                                        for k in 0..width {
                                            self.tile_mask[mask_index + k + l * 16] = None;
                                        }
                                    }

                                    i += width;
                                    mask_index += width;
                                } else {
                                    i += 1;
                                    mask_index += 1;
                                }
                            }
                        }
                    }
                }
            }
        }

        log::debug!(
            "Meshed a chunk {}us -- {} vertices {} indices",
            start.elapsed().as_micros(),
            mesh.vertices.len(),
            mesh.indices.len(),
        );
    }

    pub fn naive(&self, chunk: &Chunk, mesh: &mut ChunkMesh) {
        let start = Instant::now();
        mesh.indices.clear();
        mesh.vertices.clear();
        let mut index_offset = 0;
        for (cube_index, cube) in chunk
            .sections()
            .iter()
            .enumerate()
            .filter(|(_, section)| !section.is_empty())
            .map(|(index, section)| (index, section.cube().unwrap()))
        {
            let cube_offset = Vector3::new(0.0, 16.0 * cube_index as f32, 0.0);
            for (index, tile) in cube.iter_indexed() {
                let tile_id = tile.id().0;
                if tile_id == 0 {
                    continue;
                }
                let index_parts: (usize, usize, usize) = index.into();
                let tile_offset = chunk.position() + Vector3::from(index_parts) + cube_offset - 0.5;
                mesh.vertices
                    .extend(CUBE_VERTEX_POSITIONS.iter().map(|pos| ChunkVertex {
                        position: *pos + tile_offset,
                        diffuse: match tile_id {
                            1 => (0.0, 0.0, 1.0).into(),
                            2 => (1.0, 1.0, 0.0).into(),
                            _ => (1.0, 0.0, 0.0).into(),
                        },
                    }));
                mesh.indices.extend(
                    CUBE_INDICES
                        .iter()
                        .map(|index| *index + index_offset as u32),
                );
                index_offset += CUBE_VERTEX_POSITIONS.len();
            }
        }

        log::debug!(
            "Meshed a chunk {}us -- {} vertices {} indices",
            start.elapsed().as_micros(),
            mesh.vertices.len(),
            mesh.indices.len()
        );
    }
}

#[repr(C)]
#[derive(Copy, Clone, Default, Debug)]
pub struct ChunkVertex {
    pub position: Vector3,
    pub diffuse: Vector3,
}

unsafe impl bytemuck::Zeroable for ChunkVertex {}

unsafe impl bytemuck::Pod for ChunkVertex {}

#[derive(Debug, Default)]
pub struct ChunkMesh {
    vertices: Vec<ChunkVertex>,
    indices: Vec<u32>,
}

impl ChunkMesh {
    #[inline]
    pub fn vertices(&self) -> &Vec<ChunkVertex> {
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
    Vector3::new(0.0, 0.0, 1.0),
    Vector3::new(1.0, 0.0, 1.0),
    Vector3::new(0.0, 1.0, 1.0),
    Vector3::new(1.0, 1.0, 1.0),
    Vector3::new(0.0, 1.0, 0.0),
    Vector3::new(1.0, 1.0, 0.0),
    Vector3::new(0.0, 0.0, 0.0),
    Vector3::new(1.0, 0.0, 0.0),
];
