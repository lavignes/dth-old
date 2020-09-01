use std::{
    io::{self, ErrorKind, Read, Seek, SeekFrom},
    mem, u16, u32,
};

use crate::{collections::PoolId, math::Vector2, util};

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Default)]
pub struct BitmapId(pub u64);

impl PoolId for BitmapId {
    fn next(&self) -> BitmapId {
        BitmapId(self.0 + 1)
    }
}

#[derive(Debug, Default)]
pub struct Bitmap {
    data: Vec<u32>,
    size: Vector2,
    bytes_per_row: usize,
}

impl Bitmap {
    #[inline]
    pub fn clear(&mut self) {
        self.data.clear();
        self.size = Vector2::default();
        self.bytes_per_row = 0;
    }

    #[inline]
    pub fn data(&self) -> &[u32] {
        &self.data
    }

    #[inline]
    pub fn size(&self) -> Vector2 {
        self.size
    }

    #[inline]
    pub fn bytes_per_row(&self) -> usize {
        self.bytes_per_row
    }
}

#[derive(Debug, Default)]
pub struct BitmapReader {}

impl BitmapReader {
    /// Read a 32-bit Microsoft Bitmap file
    pub fn read_into<R: Read + Seek>(reader: &mut R, bitmap: &mut Bitmap) -> io::Result<()> {
        reader.seek(SeekFrom::Start(0))?;

        let expected_magic = u16::from_le_bytes([b'B', b'M']);
        let magic = util::read_u16(reader)?;
        if magic != expected_magic {
            return util::io_err(
                ErrorKind::InvalidData,
                format!(
                    "Expected a 'BM' ({:02X}) instead found {:02X}",
                    expected_magic, magic
                ),
            );
        }

        reader.seek(SeekFrom::Start(0x0A))?;
        let offset = util::read_u32(reader)?;

        reader.seek(SeekFrom::Start(0x12))?;
        let width = util::read_u32(reader)? as usize;
        let height = util::read_u32(reader)? as usize;

        reader.seek(SeekFrom::Start(0x1C))?;
        let color_depth = util::read_u16(reader)?;
        if color_depth != 32 {
            return util::io_err(
                ErrorKind::InvalidData,
                format!(
                    "Expected a bitmap with a 32 BPP color-depth found {} BPP instead",
                    color_depth
                ),
            );
        }

        let compression = util::read_u32(reader)?;
        // BI_RGB (0) or BI_BITFIELDS (3) are supported
        if compression != 0 && compression != 3 {
            return util::io_err(
                ErrorKind::InvalidData,
                "Compressed bitmaps are not supported",
            );
        }

        reader.seek(SeekFrom::Start(offset as u64))?;
        bitmap.clear();
        for _ in 0..(width * height) {
            bitmap.data.push(util::read_u32(reader)?);
        }
        bitmap.size = (width as f32, height as f32).into();
        bitmap.bytes_per_row = mem::size_of::<u32>() * width;

        Ok(())
    }
}
