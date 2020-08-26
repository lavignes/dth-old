use crate::{collections::CubeMap32, math::Vector2, tile::TileState};

#[derive(Debug, Default)]
pub struct ChunkSection {
    cube: CubeMap32<TileState>,
}

impl ChunkSection {
    #[inline]
    pub fn filled(tile_state: TileState) -> ChunkSection {
        ChunkSection {
            cube: CubeMap32::filled(tile_state),
        }
    }

    #[inline]
    pub fn cube(&self) -> &CubeMap32<TileState> {
        &self.cube
    }
}

#[derive(Debug, Default)]
pub struct Chunk {
    position: Vector2,
    sections: [ChunkSection; 1],
}

impl Chunk {
    #[inline]
    pub fn position(&self) -> Vector2 {
        self.position
    }

    #[inline]
    pub fn sections(&self) -> &[ChunkSection; 1] {
        &self.sections
    }
}
