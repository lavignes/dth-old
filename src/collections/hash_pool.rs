use crate::collections::XorHashMap;
use std::{
    collections::hash_map::{Iter, IterMut},
    fmt::Debug,
    hash::Hash,
};

pub trait PoolId: Hash + Eq + Debug + Default + Copy + Clone {
    /// Generate a new id using this id as a seed.
    /// Typically an id is based on an integer and this
    /// method will increment it.
    fn next(&self) -> Self;
}

/// A hashmap-based memory pool.
///
/// Rust hashmaps have a nice property that *they never implicitly shrink*.
/// This means that they can be used as an efficient sparse data-store and memory pool.
#[derive(Debug)]
pub struct HashPool<I, V>
where
    I: PoolId,
    V: Default,
{
    id: I,
    inner: XorHashMap<I, V>,
    free_list: Vec<V>,
}

impl<I, V> Default for HashPool<I, V>
where
    I: PoolId,
    V: Default,
{
    #[inline]
    fn default() -> HashPool<I, V> {
        HashPool {
            id: I::default(),
            inner: XorHashMap::default(),
            free_list: Vec::default(),
        }
    }
}

impl<I, V> HashPool<I, V>
where
    I: PoolId,
    V: Default,
{
    #[inline]
    pub fn new() -> HashPool<I, V> {
        HashPool::default()
    }

    #[inline]
    pub fn allocate(&mut self) -> (I, &V) {
        if let Some(value) = self.free_list.pop() {
            let id = self.register(value);
            return (id, self.get(id).unwrap());
        }
        let id = self.register(V::default());
        (id, self.get(id).unwrap())
    }

    #[inline]
    pub fn create<F: FnMut(I, &mut V)>(&mut self, mut factory: F) {
        let (id, _) = self.allocate();
        factory(id, self.get_mut(id).unwrap())
    }

    #[inline]
    pub fn delete(&mut self, id: &I) {
        if let Some(value) = self.inner.remove(id) {
            self.free_list.push(value);
        }
    }

    #[inline]
    pub fn register(&mut self, value: V) -> I {
        self.id = self.id.next();
        self.inner.insert(self.id, value);
        self.id
    }

    #[inline]
    pub fn get(&self, id: I) -> Option<&V> {
        self.inner.get(&id)
    }

    #[inline]
    pub fn get_mut(&mut self, id: I) -> Option<&mut V> {
        self.inner.get_mut(&id)
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
