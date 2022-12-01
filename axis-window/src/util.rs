/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use math::Zero;

/// Box type which frees its data with [`libc::free`]. This can be removed when
/// [`std::alloc::Allocator`] becomes stable.
#[cfg(feature = "libc")]
pub struct CBox<T: 'static + ?Sized> {
    data: &'static mut T,
}

#[cfg(feature = "libc")]
impl<T: 'static + Sized> CBox<T> {
    pub unsafe fn from_raw(ptr: *mut T) -> CBox<T> {
        CBox { data: &mut *ptr }
    }
}

#[cfg(feature = "libc")]
impl<T: 'static + ?Sized> std::ops::Deref for CBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

#[cfg(feature = "libc")]
impl<T: 'static + ?Sized> std::ops::DerefMut for CBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

#[cfg(feature = "libc")]
impl<T: 'static + ?Sized> Drop for CBox<T> {
    fn drop(&mut self) {
        unsafe {
            libc::free(self.data as *mut T as *mut _);
        }
    }
}

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
