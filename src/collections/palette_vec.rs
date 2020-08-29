use crate::collections::{PackedIntVec, PackedIntVecIterator};
use std::{hash::Hash, iter::FromIterator};

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
        let mut vec = PaletteVec::with_capacity(palette_capacity, len);
        vec.fill(len, value);
        vec
    }

    #[inline]
    pub fn with_capacity(palette_capacity: usize, capacity: usize) -> PaletteVec<T> {
        PaletteVec {
            palette: Vec::with_capacity(capacity),
            indices: PackedIntVec::with_capacity(
                (palette_capacity as f64).log2().ceil() as u32,
                capacity,
            ),
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.palette.clear();
        self.indices.clear();
    }

    #[inline]
    pub fn fill(&mut self, len: usize, value: T) {
        self.palette.clear();
        self.palette.push(value);
        self.indices.fill(len, 0);
    }

    #[inline]
    pub fn get(&self, index: usize) -> &T {
        &self.palette[self.indices.get(index) as usize]
    }

    /// Get a mutable ref to the value in the palette found at `index`.
    ///
    /// Mutating this ref mutates all items that share this palette value.
    #[inline]
    pub fn get_palette_mut(&mut self, index: usize) -> &mut T {
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
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    #[inline]
    pub fn palette_len(&self) -> usize {
        self.palette.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
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
        if self.palette.len() > self.indices.max_item() as usize {
            // Have to re-allocate the indices
            self.indices = self
                .indices
                .resized_copy((self.palette.len() as f64).log2().ceil() as u32);
        }
    }
}

impl<T> FromIterator<T> for PaletteVec<T>
where
    T: Eq + Default + Hash + Clone,
{
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> PaletteVec<T> {
        let iter = iter.into_iter();
        let mut vec = if let Some(hint) = iter.size_hint().1 {
            PaletteVec::with_capacity(16, hint)
        } else {
            PaletteVec::with_capacity(16, 0)
        };
        for item in iter {
            vec.push(item);
        }
        vec
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
        assert_eq!(3, p.indices.max_item());

        for i in 2..18 {
            p.push(i);
        }
        assert_eq!(18, p.palette.len());
        assert_eq!(0x1F, p.indices.max_item());
    }
}
