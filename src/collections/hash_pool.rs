use crate::collections::XorHashMap;
use std::{
    collections::hash_map::{Iter, IterMut},
    fmt::Debug,
    hash::Hash,
};

pub trait Id: Hash + Eq + Debug + Default + Copy + Clone {
    /// Generate a new Id using this Id as a seed.
    /// Typically an Id is based on an integer and this
    /// method will increment it.
    fn next(&self) -> Self;
}

/// A hashmap-based memory pool.
///
/// Rust hashmaps have a nice property that *they never implicitly shrink*.
/// This means that they can be used as an efficient sparse data-store and memory pool.
#[derive(Debug)]
pub struct HashPool<I: Id, V> {
    id: I,
    inner: XorHashMap<I, V>,
}

impl<I: Id, V> Default for HashPool<I, V> {
    #[inline]
    fn default() -> HashPool<I, V> {
        HashPool {
            id: I::default(),
            inner: XorHashMap::<I, V>::default(),
        }
    }
}

impl<I: Id, V> HashPool<I, V> {
    #[inline]
    pub fn new() -> HashPool<I, V> {
        HashPool::<I, V>::default()
    }

    #[inline]
    pub fn register(&mut self, value: V) -> I {
        self.id = self.id.next();
        self.inner.insert(self.id, value);
        self.id
    }

    #[inline]
    pub fn get(&self, id: &I) -> Option<&V> {
        self.inner.get(id)
    }

    #[inline]
    pub fn get_mut(&mut self, id: &I) -> Option<&mut V> {
        self.inner.get_mut(id)
    }

    #[inline]
    pub fn iter(&mut self) -> Iter<'_, I, V> {
        self.inner.iter()
    }

    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, I, V> {
        self.inner.iter_mut()
    }
}
