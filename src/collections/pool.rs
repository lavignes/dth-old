use std::{
    marker::PhantomData,
    mem::{self, MaybeUninit},
    slice::{Iter, IterMut},
};

#[derive(Default, Debug)]
pub struct Handle<T> {
    index: usize,
    epoch: usize,
    marker: PhantomData<T>,
}

// FIXME: https://github.com/rust-lang/rust/issues/26925
impl<T> Copy for Handle<T> {}

impl<T> Clone for Handle<T> {
    fn clone(&self) -> Handle<T> {
        *self
    }
}

#[derive(Debug)]
struct Entry<T> {
    epoch: usize,
    data: Option<T>,
}

#[derive(Default, Debug)]
pub struct Pool<T> {
    entries: Vec<Entry<T>>,
    free_list: Vec<usize>,
}

impl<T> Pool<T> {
    pub fn register(&mut self, data: T) -> Handle<T> {
        self.register_with_callback(data, |_, _| {})
    }

    /// Register data into the pool but execute a callback function
    /// once the handle has been found. This lets you store the handle
    /// back into the data itself if necessary.
    pub fn register_with_callback<F: Fn(&mut T, Handle<T>)>(
        &mut self,
        data: T,
        callback: F,
    ) -> Handle<T> {
        if let Some(index) = self.free_list.pop() {
            let entry = self.entries.get_mut(index).unwrap();
            entry.epoch += 1;
            let handle = Handle {
                index,
                epoch: entry.epoch,
                marker: PhantomData,
            };
            let mut data = data;
            callback(&mut data, handle);
            entry.data.replace(data);
            handle
        } else {
            let index = self.entries.len();
            let handle = Handle {
                index,
                epoch: 1,
                marker: PhantomData,
            };
            let mut data = data;
            callback(&mut data, handle);
            self.entries.push(Entry {
                epoch: 1,
                data: Some(data),
            });
            handle
        }
    }

    #[inline]
    pub fn remove(&mut self, handle: Handle<T>) -> T {
        self.try_remove(handle).unwrap()
    }

    pub fn try_remove(&mut self, handle: Handle<T>) -> Option<T> {
        if let Some(entry) = self.entries.get_mut(handle.index) {
            if entry.epoch != handle.epoch || entry.data.is_none() {
                return None;
            }
            self.free_list.push(handle.index);
            entry.data.take()
        } else {
            None
        }
    }

    #[inline]
    pub fn get(&self, handle: Handle<T>) -> &T {
        self.try_get(handle).unwrap()
    }

    pub fn try_get(&self, handle: Handle<T>) -> Option<&T> {
        self.entries
            .get(handle.index)
            .and_then(|entry| {
                if entry.epoch != handle.epoch || entry.data.is_none() {
                    return None;
                }
                Some(&entry.data)
            })
            .unwrap()
            .as_ref()
    }

    #[inline]
    pub fn get_mut(&mut self, handle: Handle<T>) -> &mut T {
        self.try_get_mut(handle).unwrap()
    }

    pub fn try_get_mut(&mut self, handle: Handle<T>) -> Option<&mut T> {
        self.entries
            .get_mut(handle.index)
            .and_then(|entry| {
                if entry.epoch != handle.epoch || entry.data.is_none() {
                    return None;
                }
                Some(&mut entry.data)
            })
            .unwrap()
            .as_mut()
    }

    #[inline]
    pub fn try_get_mut_arr<const N: usize>(
        &mut self,
        handles: [Handle<T>; N],
    ) -> [Option<&mut T>; N] {
        // Ensure no 2 handles have the same index
        // This function is only safe when all handles are unique
        for (i, needle) in handles.iter().enumerate() {
            for haystack in handles.iter().skip(i + 1) {
                assert_ne!(needle.index, haystack.index);
            }
        }
        // Safety: This is ok as long as we make sure to initialize every object below.
        // TODO: `MaybeUninit::uninit_array` is in nightly
        let mut result: [MaybeUninit<Option<&mut T>>; N] =
            unsafe { MaybeUninit::uninit().assume_init() };

        for (i, handle) in handles.iter().enumerate() {
            // Safety: The lifetime of all output T are the same as self.
            result[i] = MaybeUninit::new(unsafe { (*(self as *mut Self)).try_get_mut(*handle) });
        }
        // FIXME: Would rather transmute https://github.com/rust-lang/rust/issues/47966
        // Safety: Source and target arrays are the same size.
        unsafe { mem::transmute_copy(&result) }
    }

    #[inline]
    pub fn iter(&self) -> PoolIter<T> {
        PoolIter {
            inner: self.entries.iter(),
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> PoolIterMut<T> {
        PoolIterMut {
            inner: self.entries.iter_mut(),
        }
    }
}

pub struct PoolIter<'a, T: 'a> {
    inner: Iter<'a, Entry<T>>,
}

impl<'a, T> Iterator for PoolIter<'a, T> {
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<&'a T> {
        loop {
            if let Some(next) = self.inner.next() {
                if next.data.is_some() {
                    return next.data.as_ref();
                }
            } else {
                return None;
            }
        }
    }
}

pub struct PoolIterMut<'a, T: 'a> {
    inner: IterMut<'a, Entry<T>>,
}

impl<'a, T> Iterator for PoolIterMut<'a, T> {
    type Item = &'a mut T;

    #[inline]
    fn next(&mut self) -> Option<&'a mut T> {
        loop {
            if let Some(next) = self.inner.next() {
                if next.data.is_some() {
                    return next.data.as_mut();
                }
            } else {
                return None;
            }
        }
    }
}
