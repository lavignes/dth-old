use crate::{collections::CubeMap32, gfx::ChunkMesh, tile::TileState};

#[derive(Debug, Default)]
struct ChunkSection {
    cube: CubeMap32<TileState>,
}

#[derive(Debug, Default)]
struct Chunk {
    sections: [ChunkSection; 16],
    mesh: ChunkMesh,
}
