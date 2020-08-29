#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct TileId(pub u64);

impl TileId {
    pub const VOID: TileId = TileId::void();

    #[inline]
    pub const fn void() -> TileId {
        TileId(0)
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub enum TileStateFormat {
    None,
}

impl Default for TileStateFormat {
    #[inline]
    fn default() -> TileStateFormat {
        TileStateFormat::None
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TileFace {
    Front = 0,
    Back = 1,
    Right = 2,
    Left = 3,
    Top = 4,
    Bottom = 5,
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Copy, Clone)]
pub struct TileState {
    id: TileId,
    format: TileStateFormat,
}

impl TileState {
    pub const VOID: TileState = TileState::void();

    #[inline]
    pub const fn void() -> TileState {
        TileState {
            id: TileId::VOID,
            format: TileStateFormat::None,
        }
    }

    #[inline]
    pub fn new(id: TileId) -> TileState {
        TileState {
            id,
            ..TileState::default()
        }
    }

    #[inline]
    pub fn with_format(id: TileId, format: TileStateFormat) -> TileState {
        TileState { id, format }
    }

    #[inline]
    pub fn id(&self) -> TileId {
        self.id
    }

    #[inline]
    pub fn is_void(&self) -> bool {
        self.id == TileId::void()
    }
}
