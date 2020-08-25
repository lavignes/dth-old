use std::{
    collections::{HashMap, HashSet},
    hash::{BuildHasherDefault, Hasher},
};

/// A HashMap that uses the XorHasher
pub type XorHashMap<K, V> = HashMap<K, V, BuildHasherDefault<XorHasher>>;

/// A HashSet that uses the XorHasher
pub type XorHashSet<T> = HashSet<T, BuildHasherDefault<XorHasher>>;

/// A Hasher that just returns the value for any data <= 64bits
/// otherwise it just runs xor on every byte of input.
#[derive(Default)]
pub struct XorHasher(u64);

impl Hasher for XorHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        for byte in bytes.iter() {
            self.0 ^= *byte as u64;
        }
    }

    #[inline]
    fn write_u8(&mut self, i: u8) {
        self.0 = i as u64;
    }

    #[inline]
    fn write_u16(&mut self, i: u16) {
        self.0 = i as u64;
    }

    #[inline]
    fn write_u32(&mut self, i: u32) {
        self.0 = i as u64;
    }

    #[inline]
    fn write_u64(&mut self, i: u64) {
        self.0 = i as u64;
    }

    #[inline]
    fn write_i8(&mut self, i: i8) {
        self.0 = i as u64;
    }

    #[inline]
    fn write_i16(&mut self, i: i16) {
        self.0 = i as u64;
    }

    #[inline]
    fn write_i32(&mut self, i: i32) {
        self.0 = i as u64;
    }

    #[inline]
    fn write_i64(&mut self, i: i64) {
        self.0 = i as u64;
    }
}
