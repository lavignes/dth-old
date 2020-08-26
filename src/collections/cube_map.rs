use crate::collections::{PaletteVec, PaletteVecIterator};
use std::hash::Hash;

/// A cube-map of length 32
#[derive(Debug)]
pub struct CubeMap32<T>
where
    T: Eq + Default + Hash + Clone,
{
    inner: PaletteVec<T>,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct CubeMapIndex32(usize);

impl CubeMapIndex32 {
    #[inline]
    pub fn x(self) -> usize {
        self.0 & 0b11111
    }

    #[inline]
    pub fn y(self) -> usize {
        (self.0 >> 5) & 0b11111
    }

    #[inline]
    pub fn z(self) -> usize {
        self.0 >> 10
    }
}

impl From<usize> for CubeMapIndex32 {
    #[inline]
    fn from(value: usize) -> CubeMapIndex32 {
        CubeMapIndex32(value)
    }
}

impl Into<usize> for CubeMapIndex32 {
    #[inline]
    fn into(self) -> usize {
        self.0
    }
}

impl From<(usize, usize, usize)> for CubeMapIndex32 {
    #[inline]
    fn from(value: (usize, usize, usize)) -> CubeMapIndex32 {
        CubeMapIndex32(value.1 << 10 | value.2 << 5 | value.0)
    }
}

impl Into<(usize, usize, usize)> for CubeMapIndex32 {
    #[inline]
    fn into(self) -> (usize, usize, usize) {
        (self.0 & 0b11111, (self.0 >> 5) & 0b11111, self.0 >> 10)
    }
}

impl<T> CubeMap32<T>
where
    T: Eq + Default + Hash + Clone,
{
    #[inline]
    pub fn filled(value: T) -> CubeMap32<T> {
        CubeMap32 {
            inner: PaletteVec::filled(16, 32 * 32 * 32, value),
        }
    }

    #[inline]
    pub fn get(&self, index: CubeMapIndex32) -> &T {
        &self.inner.get(index.0)
    }

    #[inline]
    pub fn get_mut(&mut self, index: CubeMapIndex32) -> &mut T {
        self.inner.get_mut(index.0)
    }

    #[inline]
    pub fn set(&mut self, index: CubeMapIndex32, value: T) {
        self.inner.set(index.0, value)
    }

    #[inline]
    pub fn iter(&self) -> CubeMap32Iterator<T> {
        CubeMap32Iterator {
            inner: self.inner.iter(),
        }
    }

    #[inline]
    pub fn iter_indexed(&self) -> CubeMap32IndexIterator<T> {
        CubeMap32IndexIterator {
            inner: self.inner.iter(),
            x: 0,
            y: 0,
            z: 0,
        }
    }
}

impl<'a, T> IntoIterator for &'a CubeMap32<T>
where
    T: Eq + Default + Hash + Clone,
{
    type Item = &'a T;
    type IntoIter = CubeMap32Iterator<'a, T>;

    #[inline]
    fn into_iter(self) -> CubeMap32Iterator<'a, T> {
        self.iter()
    }
}

impl<T> Default for CubeMap32<T>
where
    T: Eq + Default + Hash + Clone,
{
    #[inline]
    fn default() -> CubeMap32<T> {
        CubeMap32::filled(Default::default())
    }
}

pub struct CubeMap32IndexIterator<'a, T>
where
    T: Eq + Default + Hash + Clone,
{
    inner: PaletteVecIterator<'a, T>,
    x: usize,
    y: usize,
    z: usize,
}

impl<'a, T> Iterator for CubeMap32IndexIterator<'a, T>
where
    T: Eq + Default + Hash + Clone,
{
    type Item = (CubeMapIndex32, &'a T);

    #[inline]
    fn next(&mut self) -> Option<(CubeMapIndex32, &'a T)> {
        let index = (self.x, self.y, self.z).into();
        self.x += 1;
        if self.x >= 32 {
            self.x = 0;
            self.z += 1;
            if self.z >= 32 {
                self.z = 0;
                self.y += 1;
            }
        }
        self.inner.next().map(|t| (index, t))
    }
}

pub struct CubeMap32Iterator<'a, T>
where
    T: Eq + Default + Hash + Clone,
{
    inner: PaletteVecIterator<'a, T>,
}

impl<'a, T> Iterator for CubeMap32Iterator<'a, T>
where
    T: Eq + Default + Hash + Clone,
{
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.inner.next()
    }
}
