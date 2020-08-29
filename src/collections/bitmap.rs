use crate::collections::{PackedIntVec, PackedIntVecIterator};

pub struct BitVec {
    inner: PackedIntVec,
}

impl BitVec {
    #[inline]
    pub fn ones(len: usize) -> BitVec {
        BitVec {
            inner: PackedIntVec::filled(1, len, 1),
        }
    }

    #[inline]
    pub fn zeros(len: usize) -> BitVec {
        BitVec {
            inner: PackedIntVec::filled(1, len, 0),
        }
    }

    #[inline]
    pub fn filled(len: usize, value: bool) -> BitVec {
        BitVec {
            inner: PackedIntVec::filled(1, len, value as u64),
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.fill(self.inner.len(), 0)
    }

    #[inline]
    pub fn get(&self, index: usize) -> bool {
        self.inner.get(index) == 0
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: bool) {
        self.inner.set(index, value as u64)
    }

    #[inline]
    pub fn iter(&self) -> BitVecIterator {
        BitVecIterator {
            inner: self.inner.iter(),
        }
    }
}

pub struct BitVecIterator<'a> {
    inner: PackedIntVecIterator<'a>,
}

impl<'a> Iterator for BitVecIterator<'a> {
    type Item = bool;

    #[inline]
    fn next(&mut self) -> Option<bool> {
        self.inner.next().map(|value| value == 0)
    }
}

impl<'a> IntoIterator for &'a BitVec {
    type Item = bool;
    type IntoIter = BitVecIterator<'a>;

    #[inline]
    fn into_iter(self) -> BitVecIterator<'a> {
        self.iter()
    }
}

pub struct BitMask2 {
    inner: PackedIntVec,
    stride: usize,
}

impl BitMask2 {
    #[inline]
    pub fn ones(width: usize, height: usize) -> BitMask2 {
        BitMask2 {
            inner: PackedIntVec::filled(1, width * height, 1),
            stride: width,
        }
    }

    #[inline]
    pub fn zeros(width: usize, height: usize) -> BitMask2 {
        BitMask2 {
            inner: PackedIntVec::filled(1, width * height, 0),
            stride: width,
        }
    }

    #[inline]
    pub fn filled(width: usize, height: usize, value: bool) -> BitMask2 {
        BitMask2 {
            inner: PackedIntVec::filled(1, width * height, value as u64),
            stride: width,
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.fill(self.inner.len(), 0)
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> bool {
        self.inner.get(x + y * self.stride) == 0
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, value: bool) {
        self.inner.set(x + y * self.stride, value as u64)
    }

    #[inline]
    pub fn iter(&self) -> BitMask2Iterator {
        BitMask2Iterator {
            inner: self.inner.iter(),
        }
    }
}

pub struct BitMask2Iterator<'a> {
    inner: PackedIntVecIterator<'a>,
}

impl<'a> Iterator for BitMask2Iterator<'a> {
    type Item = bool;

    #[inline]
    fn next(&mut self) -> Option<bool> {
        self.inner.next().map(|value| value == 0)
    }
}

impl<'a> IntoIterator for &'a BitMask2 {
    type Item = bool;
    type IntoIter = BitMask2Iterator<'a>;

    #[inline]
    fn into_iter(self) -> BitMask2Iterator<'a> {
        self.iter()
    }
}
