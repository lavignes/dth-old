use std::{
    io::{self, ErrorKind, Read, Seek, SeekFrom},
    str, u32,
};

use crate::{math::Vector2, util};
use std::slice::Iter;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BitmapFormat {
    BgraU8,
    GrayU8,
    Dxt1,
    Dxt3,
    Dxt5,
}

impl BitmapFormat {}

impl Default for BitmapFormat {
    #[inline]
    fn default() -> BitmapFormat {
        BitmapFormat::BgraU8
    }
}

#[derive(Debug, Default)]
struct MipLevel {
    start: usize,
    end: usize,
    size: Vector2,
    bytes_per_row: usize,
}

#[derive(Debug, Default)]
pub struct MipLevelView<'a> {
    data: &'a [u8],
    size: Vector2,
    bytes_per_row: usize,
}

impl<'a> MipLevelView<'a> {
    #[inline]
    pub fn data(&self) -> &[u8] {
        self.data
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
pub struct Bitmap {
    format: BitmapFormat,
    data: Vec<u8>,
    mip_levels: Vec<MipLevel>,
}

impl Bitmap {
    pub fn clear(&mut self) {
        self.data.clear();
        self.mip_levels.clear();
        self.format = BitmapFormat::default();
    }

    #[inline]
    pub fn mip_levels(&self) -> MipLevelIterator {
        MipLevelIterator {
            inner: self.mip_levels.iter(),
            data: &self.data,
        }
    }

    #[inline]
    pub fn format(&self) -> BitmapFormat {
        self.format
    }
}

pub struct MipLevelIterator<'a> {
    inner: Iter<'a, MipLevel>,
    data: &'a Vec<u8>,
}

impl<'a> Iterator for MipLevelIterator<'a> {
    type Item = MipLevelView<'a>;

    #[inline]
    fn next(&mut self) -> Option<MipLevelView<'a>> {
        self.inner.next().map(|level| MipLevelView {
            data: &self.data[level.start..level.end],
            size: level.size,
            bytes_per_row: level.bytes_per_row,
        })
    }
}

bitflags::bitflags! {
    struct PixelFormatFlags: u32 {
        const ALPHA_PIXELS = 0x00000001;
        const FOUR_CHARACTER_CODE = 0x00000004;
        const RGB = 0x00000040;
        const LUMINANCE = 0x00020000;
    }
}

bitflags::bitflags! {
    struct CapabilityFlags: u32 {
        const COMPLEX = 0x00000008;
        const TEXTURE = 0x00001000;
        const MIPMAP = 0x00400000;
    }
}

#[derive(Debug, Default)]
pub struct BitmapReader {}

impl BitmapReader {
    pub fn read_into<R: Read + Seek>(
        &mut self,
        reader: &mut R,
        bitmap: &mut Bitmap,
    ) -> io::Result<()> {
        reader.seek(SeekFrom::Start(0x00))?;

        let expected_magic = u32::from_le_bytes([b'D', b'D', b'S', b' ']);
        let magic = util::read_u32(reader)?;
        if magic != expected_magic {
            return util::io_err(
                ErrorKind::InvalidData,
                format!(
                    "Expected a 'DDS ' ({:04X}) instead found {:04X}",
                    expected_magic, magic
                ),
            );
        }

        reader.seek(SeekFrom::Start(0x0C))?;
        let height = util::read_u32(reader)?;
        let width = util::read_u32(reader)?;
        let pitch = util::read_u32(reader)?;
        reader.seek(SeekFrom::Current(0x04))?;
        let mip_levels = util::read_u32(reader)?;

        reader.seek(SeekFrom::Start(0x50))?;
        let format_flags_bytes = util::read_u32(reader)?;
        let format_flags = util::io_err_option(
            PixelFormatFlags::from_bits(format_flags_bytes),
            ErrorKind::InvalidData,
            || {
                format!(
                    "Unsupported DDS pixel format ({:04X}). The file is probably malformed",
                    format_flags_bytes
                )
            },
        )?;
        let four_character_code_bytes = util::read_u32(reader)?.to_le_bytes();
        let four_character_code = util::io_err_result(
            str::from_utf8(&four_character_code_bytes),
            ErrorKind::InvalidData,
        )?;
        let rgb_bit_counts = util::read_u32(reader)?;
        let _r_bit_mask = util::read_u32(reader)?.to_le_bytes();
        let _g_bit_mask = util::read_u32(reader)?.to_le_bytes();
        let _b_bit_mask = util::read_u32(reader)?.to_le_bytes();
        let _a_bit_mask = util::read_u32(reader)?.to_le_bytes();
        let capabilities_bytes = util::read_u32(reader)?;
        util::io_err_option(
            CapabilityFlags::from_bits(capabilities_bytes),
            ErrorKind::InvalidData,
            || {
                format!(
                    "Unsupported DDS capabilities ({:04X}). The file is probably malformed",
                    capabilities_bytes
                )
            },
        )?;

        // Jump to pixel data (it is further down on FourCharacterCode == "DX11" but we dont do it)
        reader.seek(SeekFrom::Start(0x70))?;
        if format_flags.contains(PixelFormatFlags::FOUR_CHARACTER_CODE) {
            let block_size;
            match four_character_code {
                "DXT1" => {
                    bitmap.format = BitmapFormat::Dxt1;
                    block_size = 8;
                }
                "DXT3" => {
                    bitmap.format = BitmapFormat::Dxt3;
                    block_size = 16;
                }
                "DXT5" => {
                    bitmap.format = BitmapFormat::Dxt5;
                    block_size = 16;
                }
                _ => {
                    return util::io_err(
                        ErrorKind::InvalidData,
                        format!("Unsupported compression format: {}", four_character_code),
                    );
                }
            }
            let mut offset = 0;
            for mip_level in 0..mip_levels {
                let mip_width = (width >> mip_level) as usize;
                let mip_height = (height >> mip_level) as usize;
                let mip_pitch = ((mip_width + 3) / 4).max(1) * block_size;
                // This *should* also be the same as pitch at mip_level==0
                let linear_size = mip_pitch * ((mip_height + 3) / 4).max(1);
                bitmap.data.reserve(linear_size);
                for _ in 0..linear_size {
                    bitmap.data.push(util::read_u8(reader)?);
                }
                bitmap.mip_levels.push(MipLevel {
                    start: offset,
                    end: offset + linear_size,
                    size: (mip_width as f32, mip_height as f32).into(),
                    bytes_per_row: mip_pitch,
                });
                offset += linear_size;
            }
        } else if format_flags.contains(PixelFormatFlags::LUMINANCE) {
            if format_flags.contains(PixelFormatFlags::ALPHA_PIXELS) {
                return util::io_err(
                    ErrorKind::InvalidData,
                    format!(
                        "Non 8-bit Luminance pixel formats are not supported. Image pixels are {}-bit",
                        rgb_bit_counts
                    ),
                );
            }
            bitmap.format = BitmapFormat::GrayU8;
            let mut offset = 0;
            for mip_level in 0..mip_levels {
                let mip_width = (width >> mip_level) as usize;
                let mip_height = (height >> mip_level) as usize;
                let mip_pitch = mip_width;
                let linear_size = mip_pitch * mip_height;
                bitmap.data.reserve(linear_size);
                for _ in 0..linear_size {
                    bitmap.data.push(util::read_u8(reader)?);
                }
                bitmap.mip_levels.push(MipLevel {
                    start: offset,
                    end: offset + linear_size,
                    size: (mip_width as f32, mip_height as f32).into(),
                    bytes_per_row: mip_pitch,
                });
                offset += linear_size;
            }
        } else if format_flags.contains(PixelFormatFlags::RGB) {
            if rgb_bit_counts != 32 || !format_flags.contains(PixelFormatFlags::ALPHA_PIXELS) {
                return util::io_err(
                    ErrorKind::InvalidData,
                    format!(
                        "Non 32-bit RGB pixel formats are not supported. Image pixels are {}-bit",
                        rgb_bit_counts
                    ),
                );
            }
            bitmap.format = BitmapFormat::BgraU8;
            let linear_size = height * pitch;
            bitmap.data.reserve(linear_size as usize);
            for _ in 0..(width * height) {
                bitmap.data.extend(&util::read_u32(reader)?.to_le_bytes());
            }
            bitmap.mip_levels.push(MipLevel {
                start: 0,
                end: linear_size as usize,
                size: (width as f32, height as f32).into(),
                bytes_per_row: pitch as usize,
            });
        } else {
            return util::io_err(
                ErrorKind::InvalidData,
                format!("Unsupported DDS pixel format {:04X}", format_flags_bytes),
            );
        }

        Ok(())
    }
}
