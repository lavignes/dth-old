use std::mem;

const BITS_IN_U64: usize = mem::size_of::<u64>() * 8;

/// A vec-like collection that stores unsigned integers up to 64-bits in a packed format.
///
/// # Examples
///
/// With an integer size of 4 bits:
/// ```
/// use dth::collections::PackedIntVec;
/// let p = PackedIntVec::new(4);
/// assert_eq!(0b1111, p.max_item())
/// ```
///
/// The vec will pack as many 4-bit integers into a u64 as possible, meaning
/// 16 4-bit integers can fit in a given underlying cell.
///
/// It works for even non-power-of-two numbers of bits:
/// ```
/// use dth::collections::PackedIntVec;
/// let mut p = PackedIntVec::new(7);
/// assert_eq!(0b1111111, p.max_item());
///
/// p.push(42);
/// assert_eq!(42, p.get(0))
/// ```
#[derive(Debug)]
pub struct PackedIntVec {
    item_size: usize,
    max_item: u64,
    items_per_cell: usize,
    len: usize,
    inner: Vec<u64>,
}

impl PackedIntVec {
    #[inline]
    pub fn new(int_size: u32) -> PackedIntVec {
        PackedIntVec::with_capacity(int_size, 16)
    }

    #[inline]
    pub fn filled(int_size: u32, len: usize, value: u64) -> PackedIntVec {
        let mut vec = PackedIntVec::with_capacity(int_size, len);
        vec.fill(len, value);
        vec
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
        self.len = 0;
    }

    pub fn fill(&mut self, len: usize, value: u64) {
        self.clear();
        assert!(value <= self.max_item);
        self.len = len;

        let mut cell_index = 0;
        let mut cell_subindex = 0;
        for _ in 0..len {
            if cell_index >= self.inner.len() {
                self.inner.push(0);
            }
            let shift_amt = cell_subindex * self.item_size;
            self.inner[cell_index] |= value << shift_amt;

            // test for end of cell
            cell_subindex += 1;
            if cell_subindex >= self.items_per_cell {
                cell_index += 1;
                cell_subindex = 0;
            }
        }
    }

    /// Create a new packed integer vec pre-allocated for `capacity` `int_size`-bit integers.
    ///
    /// # Panics
    ///
    /// Panics if 0 >= `int_size` > 64.
    pub fn with_capacity(int_size: u32, capacity: usize) -> PackedIntVec {
        assert!(int_size <= BITS_IN_U64 as u32);
        assert_ne!(0, int_size);
        let max_item = if int_size != BITS_IN_U64 as u32 {
            2u64.pow(int_size).next_power_of_two() - 1
        } else {
            u64::max_value()
        };
        let item_size = BITS_IN_U64 - (max_item.count_zeros() as usize);
        let items_per_cell = BITS_IN_U64 / item_size;
        PackedIntVec {
            item_size,
            max_item,
            items_per_cell,
            len: 0,
            inner: Vec::with_capacity(capacity / items_per_cell),
        }
    }

    /// Create a copy of self with `new_int_size`-bit elements.
    ///
    /// # Panics
    ///
    /// * If any element in self does not fit in `new_int_size` bits.
    /// * If 0 >= `new_int_size` > 64
    ///
    #[inline]
    pub fn resized_copy(&self, new_int_size: u32) -> PackedIntVec {
        PackedIntVec::from_iter(new_int_size, self)
    }

    /// Create a new vec from an iterator.
    ///
    /// # Panics
    ///
    /// * If any element in self does not fit in `int_size` bits.
    /// * If 0 >= `int_size` > 64
    pub fn from_iter<I>(int_size: u32, iter: I) -> PackedIntVec
    where
        I: IntoIterator<Item = u64>,
    {
        let iter = iter.into_iter();
        let mut vec = if let Some(hint) = iter.size_hint().1 {
            PackedIntVec::with_capacity(int_size, hint)
        } else {
            PackedIntVec::with_capacity(int_size, 0)
        };

        let mut cell_index = 0;
        let mut cell_subindex = 0;
        for value in iter {
            assert!(value <= vec.max_item);
            vec.len += 1;
            if cell_index >= vec.inner.len() {
                vec.inner.push(0);
            }
            let shift_amt = cell_subindex * vec.item_size;
            vec.inner[cell_index] |= value << shift_amt;

            // test for end of cell
            cell_subindex += 1;
            if cell_subindex >= vec.items_per_cell {
                cell_index += 1;
                cell_subindex = 0;
            }
        }
        vec
    }

    #[inline]
    pub fn iter(&self) -> PackedIntVecIterator {
        PackedIntVecIterator {
            inner: self,
            cell_index: 0,
            cell_subindex: 0,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    #[inline]
    pub fn max_item(&self) -> u64 {
        self.max_item
    }

    pub fn get(&self, index: usize) -> u64 {
        let cell_index = index / self.items_per_cell;
        let cell_subindex = index % self.items_per_cell;

        let cell = self.inner[cell_index];
        let shift_amt = cell_subindex * self.item_size;
        (cell >> shift_amt) & self.max_item
    }

    /// Set an element at `index` to `value`.
    ///
    /// # Panics
    ///
    /// Panics if `value` > `self.max_item()`.
    pub fn set(&mut self, index: usize, value: u64) {
        assert!(value <= self.max_item);
        let cell_index = index / self.items_per_cell;
        let cell_subindex = index % self.items_per_cell;

        let cell = self.inner[cell_index];
        let shift_amt = cell_subindex * self.item_size;
        let zeroed = !(self.max_item << shift_amt) & cell;
        self.inner[cell_index] = zeroed | (value << shift_amt);
    }

    pub fn push(&mut self, value: u64) {
        let index = self.len;
        self.len += 1;
        let cell_index = index / self.items_per_cell;
        if cell_index >= self.inner.len() {
            self.inner.push(0)
        }
        let cell_subindex = index % self.items_per_cell;

        let cell = self.inner[cell_index];
        let shift_amt = cell_subindex * self.item_size;
        let zeroed = !(self.max_item << shift_amt) & cell;
        self.inner[cell_index] = zeroed | (value << shift_amt);
    }
}

impl<'a> IntoIterator for &'a PackedIntVec {
    type Item = u64;
    type IntoIter = PackedIntVecIterator<'a>;

    #[inline]
    fn into_iter(self) -> PackedIntVecIterator<'a> {
        self.iter()
    }
}

pub struct PackedIntVecIterator<'a> {
    inner: &'a PackedIntVec,
    cell_index: usize,
    cell_subindex: usize,
}

impl<'a> Iterator for PackedIntVecIterator<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        if self.cell_index >= self.inner.inner.len() {
            return None;
        }
        let cell = self.inner.inner[self.cell_index];
        let shift_amt = self.cell_subindex * self.inner.item_size;
        let value = Some((cell >> shift_amt) & self.inner.max_item);

        // test for end of cell
        self.cell_subindex += 1;
        if self.cell_subindex >= self.inner.items_per_cell {
            self.cell_index += 1;
            self.cell_subindex = 0;
        }
        value
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (
            0,
            Some(
                self.inner.len
                    - ((self.cell_index * self.inner.items_per_cell) + self.cell_subindex),
            ),
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn max_items() {
        let p = PackedIntVec::new(1);
        assert_eq!(0x01, p.max_item);

        let p = PackedIntVec::new(2);
        assert_eq!(0x03, p.max_item);

        let p = PackedIntVec::new(4);
        assert_eq!(0x0F, p.max_item);

        let p = PackedIntVec::new(5);
        assert_eq!(0x1F, p.max_item);

        let p = PackedIntVec::new(64);
        assert_eq!(0xFFFF_FFFF_FFFF_FFFF, p.max_item);
    }

    #[test]
    fn pushes() {
        let mut p = PackedIntVec::new(4);
        p.push(0x3);
        assert_eq!(0x0000_0000_0000_0003, p.inner[0]);
        p.push(0xF);
        assert_eq!(0x0000_0000_0000_00F3, p.inner[0]);
        p.push(0xA);
        assert_eq!(0x0000_0000_0000_0AF3, p.inner[0]);
        p.push(0xD);
        assert_eq!(0x0000_0000_0000_DAF3, p.inner[0]);
    }

    #[test]
    fn pushes_overflow() {
        let mut p = PackedIntVec::new(5);
        for i in 0..16 {
            p.push(i);
        }
        assert_eq!(0x05A9_2839_8A41_8820, p.inner[0]);
        assert_eq!(0x0000_0000_0007_B9AC, p.inner[1]);
    }

    #[test]
    fn sets() {
        let mut p = PackedIntVec::new(4);
        for i in 0..16 {
            p.push(i);
        }
        assert_eq!(0xFEDC_BA98_7654_3210, p.inner[0]);

        p.set(0, 0xF);
        assert_eq!(0xFEDC_BA98_7654_321F, p.inner[0]);
        p.set(1, 0xE);
        assert_eq!(0xFEDC_BA98_7654_32EF, p.inner[0]);
        p.set(2, 0xE);
        assert_eq!(0xFEDC_BA98_7654_3EEF, p.inner[0]);
        p.set(3, 0xB);
        assert_eq!(0xFEDC_BA98_7654_BEEF, p.inner[0]);
    }

    #[test]
    fn gets() {
        let mut p = PackedIntVec::new(4);
        for i in 0..16 {
            p.push(i);
        }
        assert_eq!(0xFEDC_BA98_7654_3210, p.inner[0]);

        for i in 0..16 {
            assert_eq!(i as u64, p.get(i));
        }

        for (i, item) in p.iter().enumerate() {
            assert_eq!(i as u64, item);
        }
    }

    #[test]
    fn resized_copies() {
        let mut p = PackedIntVec::new(4);
        for i in 0..16 {
            p.push(i);
        }
        assert_eq!(0xFEDC_BA98_7654_3210, p.inner[0]);

        let p = p.resized_copy(5);
        assert_eq!(0x05A9_2839_8A41_8820, p.inner[0]);
        assert_eq!(0x0000_0000_0007_B9AC, p.inner[1]);
    }
}
