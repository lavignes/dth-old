use crate::{
    collections::{PaletteVec, PaletteVecIterator},
    math::Vector3,
};
use std::{fmt::Debug, hash::Hash, iter::FromIterator};

/// A cube-map of length 16
#[derive(Debug)]
pub struct CubeMap16<T>
where
    T: Eq + Default + Hash + Clone,
{
    inner: PaletteVec<T>,
}

#[derive(Copy, Debug, Clone, Default, Eq, PartialEq, Hash)]
pub struct CubeMapIndex16(usize);

impl CubeMapIndex16 {
    #[inline]
    pub fn x(self) -> usize {
        self.0 & 0b1111
    }

    #[inline]
    pub fn y(self) -> usize {
        self.0 >> 8
    }

    #[inline]
    pub fn z(self) -> usize {
        (self.0 >> 4) & 0b1111
    }
}

impl From<usize> for CubeMapIndex16 {
    #[inline]
    fn from(value: usize) -> CubeMapIndex16 {
        CubeMapIndex16(value)
    }
}

impl From<(usize, usize, usize)> for CubeMapIndex16 {
    #[inline]
    fn from(value: (usize, usize, usize)) -> CubeMapIndex16 {
        CubeMapIndex16(value.1 << 8 | value.2 << 4 | value.0)
    }
}

impl From<Vector3> for CubeMapIndex16 {
    #[inline]
    fn from(value: Vector3) -> CubeMapIndex16 {
        CubeMapIndex16((value.y() as usize) << 8 | (value.z() as usize) << 4 | (value.x() as usize))
    }
}

impl Into<(usize, usize, usize)> for CubeMapIndex16 {
    #[inline]
    fn into(self) -> (usize, usize, usize) {
        (self.0 & 0b1111, self.0 >> 8, (self.0 >> 4) & 0b1111)
    }
}

impl<T> CubeMap16<T>
where
    T: Eq + Default + Hash + Clone,
{
    #[inline]
    pub fn filled(value: T) -> CubeMap16<T> {
        CubeMap16 {
            inner: PaletteVec::filled(16, 16 * 16 * 16, value),
        }
    }

    #[inline]
    pub fn get(&self, index: CubeMapIndex16) -> &T {
        &self.inner.get(index.0)
    }

    /// Get a mutable ref to all identical items found at `index`.
    ///
    /// Mutating this ref mutates all items that share this value.
    #[inline]
    pub fn get_identical_mut(&mut self, index: CubeMapIndex16) -> &mut T {
        self.inner.get_palette_mut(index.0)
    }

    #[inline]
    pub fn replace_identical(&mut self, index: CubeMapIndex16, value: T) {
        *self.inner.get_palette_mut(index.0) = value;
    }

    #[inline]
    pub fn set(&mut self, index: CubeMapIndex16, value: T) {
        self.inner.set(index.0, value)
    }

    #[inline]
    pub fn iter(&self) -> CubeMap16Iterator<T> {
        CubeMap16Iterator {
            inner: self.inner.iter(),
        }
    }

    #[inline]
    pub fn iter_indexed(&self) -> CubeMap16IndexIterator<T> {
        CubeMap16IndexIterator {
            inner: self.inner.iter(),
            x: 0,
            y: 0,
            z: 0,
        }
    }

    #[inline]
    pub fn palette(&self) -> &PaletteVec<T> {
        &self.inner
    }

    #[inline]
    pub fn fill(&mut self, value: T) {
        self.inner.fill(16 * 16 * 16, value);
    }
}

impl<T> FromIterator<T> for CubeMap16<T>
where
    T: Eq + Default + Hash + Clone,
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> CubeMap16<T> {
        CubeMap16 {
            inner: PaletteVec::from_iter(iter),
        }
    }
}

impl<'a, T> IntoIterator for &'a CubeMap16<T>
where
    T: Eq + Default + Hash + Clone,
{
    type Item = &'a T;
    type IntoIter = CubeMap16Iterator<'a, T>;

    #[inline]
    fn into_iter(self) -> CubeMap16Iterator<'a, T> {
        self.iter()
    }
}

impl<T> Default for CubeMap16<T>
where
    T: Eq + Default + Hash + Clone,
{
    #[inline]
    fn default() -> CubeMap16<T> {
        CubeMap16::filled(Default::default())
    }
}

pub struct CubeMap16IndexIterator<'a, T>
where
    T: Eq + Default + Hash + Clone,
{
    inner: PaletteVecIterator<'a, T>,
    x: usize,
    y: usize,
    z: usize,
}

impl<'a, T> Iterator for CubeMap16IndexIterator<'a, T>
where
    T: Eq + Default + Hash + Clone,
{
    type Item = (CubeMapIndex16, &'a T);

    #[inline]
    fn next(&mut self) -> Option<(CubeMapIndex16, &'a T)> {
        let index = (self.x, self.y, self.z).into();
        self.x += 1;
        if self.x >= 16 {
            self.x = 0;
            self.z += 1;
            if self.z >= 16 {
                self.z = 0;
                self.y += 1;
            }
        }
        self.inner.next().map(|t| (index, t))
    }
}

pub struct CubeMap16Iterator<'a, T>
where
    T: Eq + Default + Hash + Clone,
{
    inner: PaletteVecIterator<'a, T>,
}

impl<'a, T> Iterator for CubeMap16Iterator<'a, T>
where
    T: Eq + Default + Hash + Clone,
{
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.inner.next()
    }
}
