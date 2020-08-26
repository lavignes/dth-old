#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct TileId(pub u64);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum TileStateFormat {
    None,
}

impl Default for TileStateFormat {
    #[inline]
    fn default() -> TileStateFormat {
        TileStateFormat::None
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct TileState {
    id: TileId,
    format: TileStateFormat,
}
