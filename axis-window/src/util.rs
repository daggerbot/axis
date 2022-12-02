/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use math::Zero;

/// Object which invokes a callback from its destructor.
pub struct Finally<F: FnMut()> {
    f: F,
}

impl<F: FnMut()> Finally<F> {
    pub fn new(f: F) -> Finally<F> {
        Finally { f }
    }
}

impl<F: FnMut()> Drop for Finally<F> {
    fn drop(&mut self) {
        (self.f)();
    }
}

#[cfg(feature = "libc")]
mod libc {
    use std::fmt::{Debug, Formatter};

    /// Box type which frees its data with [`libc::free`]. This can be removed when
    /// [`std::alloc::Allocator`] becomes stable.
    pub struct CBox<T: 'static + ?Sized> {
        data: &'static mut T,
    }

    impl<T: 'static + Sized> CBox<T> {
        pub fn as_mut_ptr(&mut self) -> *mut T {
            self.data as *mut T
        }

        pub fn as_ptr(&self) -> *const T {
            self.data as *const T
        }

        pub unsafe fn from_raw(ptr: *mut T) -> CBox<T> {
            CBox { data: &mut *ptr }
        }
    }

    impl<T: 'static + ?Sized> AsRef<T> for CBox<T> {
        fn as_ref(&self) -> &T {
            self.data
        }
    }

    impl<T: 'static + ?Sized + Debug> Debug for CBox<T> {
        fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
            Debug::fmt(self.data, fmt)
        }
    }

    impl<T: 'static + ?Sized> std::ops::Deref for CBox<T> {
        type Target = T;
        fn deref(&self) -> &T {
            self.data
        }
    }

    impl<T: 'static + ?Sized> std::ops::DerefMut for CBox<T> {
        fn deref_mut(&mut self) -> &mut T {
            self.data
        }
    }

    impl<T: 'static + ?Sized> Drop for CBox<T> {
        fn drop(&mut self) {
            unsafe {
                libc::free(self.data as *mut T as *mut _);
            }
        }
    }
}

#[cfg(feature = "libc")]
pub use self::libc::*;

/// Clamps a value within a range. The result is undefined if `min` > `max`.
pub fn clamp<T: Ord>(x: T, min: T, max: T) -> T {
    if x < min {
        min
    } else if x > max {
        max
    } else {
        x
    }
}

/// Generic implementation of ISO C's `strlen()`.
pub unsafe fn strlen<T: Eq + Zero>(mut ptr: *const T) -> usize {
    if ptr.is_null() {
        return 0;
    }
    let mut len = 0;
    let zero = T::zero();
    while *ptr != zero {
        len += 1;
        ptr = (ptr as usize + std::mem::size_of::<T>()) as *const T;
    }
    len
}
