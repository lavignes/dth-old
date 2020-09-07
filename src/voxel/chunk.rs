use crate::{
    collections::CubeMap16,
    math::Vector3,
    tile::{TileId, TileState},
};
use std::iter::FromIterator;

use rand::{self, Rng};

#[derive(Debug, Default)]
pub struct ChunkSection {
    cube: Option<CubeMap16<TileState>>,
}

impl ChunkSection {
    #[inline]
    pub fn void() -> ChunkSection {
        ChunkSection { cube: None }
    }

    #[inline]
    pub fn filled(tile_state: TileState) -> ChunkSection {
        if tile_state.is_void() {
            ChunkSection::void()
        } else {
            ChunkSection {
                cube: Some(CubeMap16::filled(tile_state)),
            }
        }
    }

    #[inline]
    pub fn randomized() -> ChunkSection {
        let mut rng = rand::thread_rng();
        let distr = [
            0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 3,
        ];
        ChunkSection {
            cube: Some(CubeMap16::from_iter((0..(16 * 16 * 16)).map(|_| {
                TileState::new(TileId(distr[rng.gen_range(0, distr.len())]))
            }))),
        }
    }

    #[inline]
    pub fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        let distr = [
            0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 3,
        ];
        for index in 0..((16 * 16 * 16) as usize) {
            self.cube_mut().set(
                index.into(),
                TileState::new(TileId(distr[rng.gen_range(0, distr.len())])),
            )
        }
    }

    #[inline]
    pub fn fill(&mut self, tile_state: TileState) {
        if tile_state.is_void() {
            self.cube = None;
        } else {
            self.cube_mut().fill(tile_state);
        }
    }

    #[inline]
    pub fn cube(&self) -> Option<&CubeMap16<TileState>> {
        self.cube.as_ref()
    }

    #[inline]
    pub fn cube_mut(&mut self) -> &mut CubeMap16<TileState> {
        // TODO: We should cache cubes when not in use
        if self.cube.is_none() {
            self.cube = Some(CubeMap16::default())
        }
        self.cube.as_mut().unwrap()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cube.is_none()
    }
}

#[derive(Debug, Default)]
pub struct Chunk {
    position: Vector3,
    sections: [ChunkSection; 16],
}

impl Chunk {
    #[inline]
    pub fn randomized() -> Chunk {
        Chunk {
            sections: [
                ChunkSection::randomized(),
                ChunkSection::randomized(),
                ChunkSection::void(),
                ChunkSection::void(),
                ChunkSection::void(),
                ChunkSection::void(),
                ChunkSection::void(),
                ChunkSection::void(),
                ChunkSection::void(),
                ChunkSection::void(),
                ChunkSection::void(),
                ChunkSection::void(),
                ChunkSection::void(),
                ChunkSection::void(),
                ChunkSection::void(),
                ChunkSection::void(),
            ],
            ..Chunk::default()
        }
    }

    #[inline]
    pub fn filled(tile_state: TileState) -> Chunk {
        Chunk {
            sections: [
                ChunkSection::filled(tile_state),
                ChunkSection::filled(tile_state),
                ChunkSection::filled(tile_state),
                ChunkSection::filled(tile_state),
                ChunkSection::filled(tile_state),
                ChunkSection::filled(tile_state),
                ChunkSection::filled(tile_state),
                ChunkSection::filled(tile_state),
                ChunkSection::filled(tile_state),
                ChunkSection::filled(tile_state),
                ChunkSection::filled(tile_state),
                ChunkSection::filled(tile_state),
                ChunkSection::filled(tile_state),
                ChunkSection::filled(tile_state),
                ChunkSection::filled(tile_state),
                ChunkSection::filled(tile_state),
            ],
            ..Chunk::default()
        }
    }

    #[inline]
    pub fn fill(&mut self, tile_state: TileState) {
        for section in self.sections.iter_mut() {
            section.fill(tile_state);
        }
    }

    #[inline]
    pub fn randomize(&mut self) {
        for section in self.sections.iter_mut().take(8) {
            section.randomize();
        }
    }

    #[inline]
    pub fn set_position(&mut self, position: Vector3) {
        self.position = position;
    }

    #[inline]
    pub fn position(&self) -> Vector3 {
        self.position
    }

    #[inline]
    pub fn sections(&self) -> &[ChunkSection; 16] {
        &self.sections
    }

    #[inline]
    pub fn sections_mut(&mut self) -> &mut [ChunkSection; 16] {
        &mut self.sections
    }
}
