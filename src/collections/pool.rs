use std::{
    marker::PhantomData,
    slice::{Iter, IterMut},
};

#[derive(Debug)]
pub struct Handle<T> {
    index: usize,
    epoch: usize,
    marker: PhantomData<T>,
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
    #[inline]
    pub fn register(&mut self, data: T) -> Handle<T> {
        if let Some(index) = self.free_list.pop() {
            let entry = self.entries.get_mut(index).unwrap();
            entry.epoch += 1;
            entry.data.replace(data);
            Handle {
                index,
                epoch: entry.epoch,
                marker: PhantomData,
            }
        } else {
            let index = self.entries.len();
            self.entries.push(Entry {
                epoch: 1,
                data: Some(data),
            });
            Handle {
                index,
                epoch: 1,
                marker: PhantomData,
            }
        }
    }

    #[inline]
    pub fn remove(&mut self, handle: Handle<T>) -> T {
        self.try_remove(handle).unwrap()
    }

    #[inline]
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

    #[inline]
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

    #[inline]
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
    pub fn get_mut_2(&mut self, handles: (Handle<T>, Handle<T>)) -> (&mut T, &mut T) {
        assert_ne!(handles.0.index, handles.1.index);
        unsafe {
            let s = self as *mut Self;
            (
                (*s).try_get_mut(handles.0).unwrap(),
                (*s).try_get_mut(handles.1).unwrap(),
            )
        }
    }

    #[inline]
    pub fn get_mut_3(
        &mut self,
        handles: (Handle<T>, Handle<T>, Handle<T>),
    ) -> (&mut T, &mut T, &mut T) {
        assert_ne!(handles.0.index, handles.1.index);
        assert_ne!(handles.0.index, handles.2.index);
        assert_ne!(handles.1.index, handles.2.index);
        unsafe {
            let s = self as *mut Self;
            (
                (*s).try_get_mut(handles.0).unwrap(),
                (*s).try_get_mut(handles.1).unwrap(),
                (*s).try_get_mut(handles.2).unwrap(),
            )
        }
    }

    #[inline]
    pub fn get_mut_4(
        &mut self,
        handles: (Handle<T>, Handle<T>, Handle<T>, Handle<T>),
    ) -> (&mut T, &mut T, &mut T, &mut T) {
        assert_ne!(handles.0.index, handles.1.index);
        assert_ne!(handles.0.index, handles.2.index);
        assert_ne!(handles.0.index, handles.3.index);
        assert_ne!(handles.1.index, handles.2.index);
        assert_ne!(handles.1.index, handles.3.index);
        assert_ne!(handles.2.index, handles.3.index);
        unsafe {
            let s = self as *mut Self;
            (
                (*s).try_get_mut(handles.0).unwrap(),
                (*s).try_get_mut(handles.1).unwrap(),
                (*s).try_get_mut(handles.2).unwrap(),
                (*s).try_get_mut(handles.3).unwrap(),
            )
        }
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
