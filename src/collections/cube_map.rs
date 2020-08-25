use crate::collections::{PaletteVec, PaletteVecIterator};

/// A cube-map of length 32
#[derive(Debug)]
pub struct CubeMap32<T>
where
    T: Eq + Default,
{
    inner: PaletteVec<T>,
}

#[inline]
const fn cube32_index(x: usize, y: usize, z: usize) -> usize {
    y << 10 | z << 5 | x
}

impl<T> CubeMap32<T>
where
    T: Eq + Default,
{
    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> &T {
        &self.inner.get(cube32_index(x, y, z))
    }

    #[inline]
    pub fn get_mut(&mut self, x: usize, y: usize, z: usize) -> &mut T {
        self.inner.get_mut(cube32_index(x, y, z))
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, value: T) {
        self.inner.set(cube32_index(x, y, z), value)
    }

    #[inline]
    pub fn iter(&self) -> CubeMap32Iterator<T> {
        CubeMap32Iterator {
            inner: self.inner.iter(),
        }
    }
}

impl<'a, T> IntoIterator for &'a CubeMap32<T>
where
    T: Eq + Default,
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
    T: Eq + Default,
{
    #[inline]
    fn default() -> CubeMap32<T> {
        CubeMap32 {
            inner: PaletteVec::with_capacity(16, 32 * 32 * 32),
        }
    }
}

pub struct CubeMap32Iterator<'a, T>
where
    T: Eq + Default,
{
    inner: PaletteVecIterator<'a, T>,
}

impl<'a, T> Iterator for CubeMap32Iterator<'a, T>
where
    T: Eq + Default,
{
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        self.inner.next()
    }
}
