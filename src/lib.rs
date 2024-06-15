#![doc = include_str!("../README.md")]

#![no_std]

use core::mem::{ManuallyDrop, MaybeUninit};

extern crate alloc;
use alloc::boxed::Box;

/// A type with a pre-specified size, regardless of the size of the T type contained
pub union LocalOrHeap<T, SizeT = [u8; 16]> {
    heap: ManuallyDrop<Box<T>>,
    local: ManuallyDrop<MaybeUninit<SizeT>>,
    _align: ManuallyDrop<[T; 0]>,
}

impl<T, SizeT> LocalOrHeap<T, SizeT> {
    /// Creates a new `LocalOrHeap` with the provided inner value
    #[inline]
    pub fn new(val: T) -> Self {
        if Self::is_heap() {
            Self{ heap: ManuallyDrop::new(Box::new(val))}
        } else {
            let mut storage = MaybeUninit::<SizeT>::uninit();
            let size = core::mem::size_of::<T>();
            // SAFETY: We know we won't overwrite storage, and we also know storage
            // will be aligned at least as coarsely as T
            unsafe{ core::ptr::copy_nonoverlapping(&val as *const T as *const u8, storage.as_mut_ptr() as *mut u8, size); }
            core::mem::forget(val);
            Self{ local: ManuallyDrop::new(storage) }
        }
    }

    /// Consumes the `LocalOrHeap`, returning the contained inner value
    #[inline]
    pub fn into_inner(mut this: Self) -> T {
        let t = if Self::is_heap() {
            // SAFETY: We know we have a `heap` because of the size_of T
            unsafe{ *ManuallyDrop::take(&mut this.heap) }
        } else {
            let mut t = MaybeUninit::<T>::uninit();
            // SAFETY: We know we have a `local` because of the size_of T
            unsafe{
                let storage_ptr = this.local.as_ptr().cast::<T>();
                core::ptr::copy_nonoverlapping(storage_ptr, t.as_mut_ptr() as *mut T, 1);
                t.assume_init()
            }
        };
        core::mem::forget(this);
        t
    }

    /// Returns `true` if the `LocalOrHeap` will use heap storage
    #[inline]
    pub fn is_heap() -> bool {
        core::mem::size_of::<T>() > core::mem::size_of::<SizeT>()
    }

}

impl<T, SizeT> Drop for LocalOrHeap<T, SizeT> {
    fn drop(&mut self) {
        if Self::is_heap() {
            // SAFETY: We know we have a `heap` because of the size_of T
            unsafe{ ManuallyDrop::drop(&mut self.heap) };
        } else {
            // SAFETY: We know we have a `local` because of the size_of T
            unsafe{ core::ptr::drop_in_place::<T>((self.local.as_mut_ptr()).cast()); }
        }
    }
}

impl<T, SizeT> core::ops::Deref for LocalOrHeap<T, SizeT> {
    type Target = T;

    fn deref(&self) -> &T {
        if Self::is_heap() {
            // SAFETY: We know we have a `heap` because of the size_of T
            unsafe{ &self.heap }
        } else {
            // SAFETY: We know we have a `local` because of the size_of T
            unsafe{ &*self.local.as_ptr().cast() }
        }
    }
}

impl<T, SizeT> core::ops::DerefMut for LocalOrHeap<T, SizeT> {
    fn deref_mut(&mut self) -> &mut T {
        if Self::is_heap() {
            // SAFETY: We know we have a `heap` because of the size_of T
            unsafe{ &mut self.heap }
        } else {
            // SAFETY: We know we have a `local` because of the size_of T
            unsafe{ &mut *self.local.as_mut_ptr().cast() }
        }
    }
}

impl<T: Clone, SizeT> Clone for LocalOrHeap<T, SizeT> {
    fn clone(&self) -> Self {
        let t: T = (&**self).clone();
        Self::new(t)
    }
}

impl<T: core::fmt::Display, SizeT> core::fmt::Display for LocalOrHeap<T, SizeT> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Display::fmt(&**self, f)
    }
}

impl<T: core::fmt::Debug, SizeT> core::fmt::Debug for LocalOrHeap<T, SizeT> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        core::fmt::Debug::fmt(&**self, f)
    }
}

impl<T, SizeT> core::fmt::Pointer for LocalOrHeap<T, SizeT> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let ptr: *const T = &**self;
        core::fmt::Pointer::fmt(&ptr, f)
    }
}

impl<T: PartialEq, SizeT> PartialEq for LocalOrHeap<T, SizeT> {
    fn eq(&self, other: &Self) -> bool {
        PartialEq::eq(&**self, &**other)
    }
}

impl<T: Eq, SizeT> Eq for LocalOrHeap<T, SizeT> {}

impl<T: PartialOrd, SizeT> PartialOrd for LocalOrHeap<T, SizeT> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
    fn lt(&self, other: &Self) -> bool {
        PartialOrd::lt(&**self, &**other)
    }
    fn le(&self, other: &Self) -> bool {
        PartialOrd::le(&**self, &**other)
    }
    fn ge(&self, other: &Self) -> bool {
        PartialOrd::ge(&**self, &**other)
    }
    fn gt(&self, other: &Self) -> bool {
        PartialOrd::gt(&**self, &**other)
    }
}

impl<T: Ord, SizeT> Ord for LocalOrHeap<T, SizeT> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        Ord::cmp(&**self, &**other)
    }
}

impl<T: core::hash::Hash, SizeT> core::hash::Hash for LocalOrHeap<T, SizeT> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl<T, SizeT> AsRef<T> for LocalOrHeap<T, SizeT> {
    fn as_ref(&self) -> &T {
        &*self
    }
}

unsafe impl<T: Send, SizeT> Send for LocalOrHeap<T, SizeT> {}
unsafe impl<T: Sync, SizeT> Sync for LocalOrHeap<T, SizeT> {}

#[cfg(test)]
mod tests {
    extern crate std;
    use alloc::boxed::Box;
    use std::println;
    use crate::LocalOrHeap;

    #[test]
    fn local_or_heap_test() {
        assert_eq!(core::mem::align_of::<LocalOrHeap<u128>>(), core::mem::align_of::<u128>());
        assert_eq!(core::mem::align_of::<LocalOrHeap<u16>>(), core::mem::align_of::<Box<usize>>());

        assert_eq!(core::mem::size_of::<LocalOrHeap<u8>>(), core::mem::size_of::<[u8; 16]>());
        assert_eq!(core::mem::size_of::<LocalOrHeap<[u8; 1024]>>(), core::mem::size_of::<[u8; 16]>());

        let int_obj = LocalOrHeap::<usize>::new(42);
        assert_eq!(LocalOrHeap::<usize>::is_heap(), false);
        let mut int_obj_clone = int_obj.clone();

        println!("{int_obj}");
        println!("{int_obj:p}");
        assert_eq!(&*int_obj, &42);
        assert_eq!(LocalOrHeap::into_inner(int_obj), 42);

        assert_eq!(&*int_obj_clone, &42);
        *int_obj_clone = 12345;
        assert_eq!(LocalOrHeap::into_inner(int_obj_clone), 12345);

        let buf_obj = LocalOrHeap::<[usize; 8]>::new([0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(LocalOrHeap::<[usize; 8]>::is_heap(), true);
        let mut buf_obj_clone = buf_obj.clone();

        println!("{buf_obj:?}");
        println!("{buf_obj:p}");
        assert_eq!(buf_obj.as_ref(), &[0, 1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(LocalOrHeap::into_inner(buf_obj), [0, 1, 2, 3, 4, 5, 6, 7]);

        assert_eq!(&*buf_obj_clone, &[0, 1, 2, 3, 4, 5, 6, 7]);
        *buf_obj_clone = [7, 6, 5, 4, 3, 2, 1, 0];
        assert_eq!(LocalOrHeap::into_inner(buf_obj_clone), [7, 6, 5, 4, 3, 2, 1, 0]);

    }
}
