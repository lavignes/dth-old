use crate::{
    collections::{PackedIntVec, PackedIntVecIterator},
    math::IntLog2,
};
use std::hash::Hash;

#[derive(Debug)]
pub struct PaletteVec<T>
where
    T: Eq + Default + Hash + Clone,
{
    palette: Vec<T>,
    indices: PackedIntVec,
}

/// A compressed vec-like collection of `T`.
///
/// Conceptually, a `PaletteVec` is a list of unique elements called a "palette"
/// and a list of indices that point to elements in the palette.
impl<T> PaletteVec<T>
where
    T: Eq + Default + Hash + Clone,
{
    #[inline]
    pub fn filled(palette_capacity: usize, len: usize, value: T) -> PaletteVec<T> {
        PaletteVec {
            palette: vec![value],
            indices: PackedIntVec::filled((palette_capacity as u64).log2() as u32, len, 0),
        }
    }

    #[inline]
    pub fn with_capacity(palette_capacity: usize, capacity: usize) -> PaletteVec<T> {
        PaletteVec {
            palette: Vec::with_capacity(capacity),
            indices: PackedIntVec::with_capacity((palette_capacity as u64).log2() as u32, capacity),
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> &T {
        &self.palette[self.indices.get(index) as usize]
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> &mut T {
        &mut self.palette[self.indices.get(index) as usize]
    }

    pub fn set(&mut self, index: usize, value: T) {
        let palette_index = self.palette.iter().position(|t| t.eq(&value));
        if let Some(palette_index) = palette_index {
            self.indices.set(index, palette_index as u64);
        } else {
            self.palette.push(value);
            self.try_grow_palette();
            self.indices.set(index, (self.palette.len() - 1) as u64);
        }
    }

    pub fn push(&mut self, value: T) {
        let palette_index = self.palette.iter().position(|t| t.eq(&value));
        if let Some(palette_index) = palette_index {
            self.indices.push(palette_index as u64);
        } else {
            self.palette.push(value);
            self.try_grow_palette();
            self.indices.push((self.palette.len() - 1) as u64);
        }
    }

    #[inline]
    pub fn iter(&self) -> PaletteVecIterator<T> {
        PaletteVecIterator {
            inner: self,
            inner_iter: self.indices.iter(),
        }
    }

    #[inline]
    fn try_grow_palette(&mut self) {
        // The palette is full! :(
        if self.palette.len() >= self.indices.max_item() as usize {
            // Have to re-allocate the indices
            self.indices = self
                .indices
                .resized_copy((self.palette.capacity() as u64).log2() as u32);
        }
    }
}

impl<'a, T> IntoIterator for &'a PaletteVec<T>
where
    T: Eq + Default + Hash + Clone,
{
    type Item = &'a T;
    type IntoIter = PaletteVecIterator<'a, T>;

    #[inline]
    fn into_iter(self) -> PaletteVecIterator<'a, T> {
        self.iter()
    }
}

impl<T> Default for PaletteVec<T>
where
    T: Eq + Default + Hash + Clone,
{
    #[inline]
    fn default() -> PaletteVec<T> {
        PaletteVec::with_capacity(4, 16)
    }
}

pub struct PaletteVecIterator<'a, T>
where
    T: Eq + Default + Hash + Clone,
{
    inner: &'a PaletteVec<T>,
    inner_iter: PackedIntVecIterator<'a>,
}

impl<'a, T> Iterator for PaletteVecIterator<'a, T>
where
    T: Eq + Default + Hash + Clone,
{
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        if let Some(index) = self.inner_iter.next() {
            Some(&self.inner.palette[index as usize])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn sanity_tests() {
        let mut p = PaletteVec::default();
        for i in 0..2 {
            p.push(i);
        }
        assert_eq!(2, p.palette.len());
        // there is a minimum of 4-bits
        assert_eq!(0xF, p.indices.max_item());

        for i in 2..18 {
            p.push(i);
        }
        assert_eq!(18, p.palette.len());
        assert_eq!(0x1F, p.indices.max_item());
    }
}
