/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::ops::{Deref, DerefMut, Index, IndexMut};

use crate::driver::win32::error::Win32Error;

/// Boxed type which uses `LocalFree` to free its data.
/// This will no longer be needed when [`std::alloc::Allocator`] becomes standardized.
pub struct LocalBox<T: 'static + ?Sized> {
    data: &'static mut T,
}

impl<T: 'static + Sized> LocalBox<T> {
    pub unsafe fn from_raw(ptr: *mut T) -> LocalBox<T> {
        LocalBox { data: &mut *ptr }
    }
}

impl<T: 'static + Sized> LocalBox<[T]> {
    pub unsafe fn from_raw_parts(ptr: *mut T, len: usize) -> LocalBox<[T]> {
        LocalBox {
            data: std::slice::from_raw_parts_mut(ptr, len),
        }
    }
}

impl<T: 'static + ?Sized> Deref for LocalBox<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl<T: 'static + ?Sized> DerefMut for LocalBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

impl<T: 'static + ?Sized> Drop for LocalBox<T> {
    fn drop(&mut self) {
        unsafe {
            winapi::um::winbase::LocalFree(self.data as *mut T as *mut _);
        }
    }
}

impl<I, T: 'static + ?Sized + Index<I>> Index<I> for LocalBox<T> {
    type Output = T::Output;
    fn index(&self, index: I) -> &T::Output {
        self.data.index(index)
    }
}

impl<I, T: 'static + ?Sized + IndexMut<I>> IndexMut<I> for LocalBox<T> {
    fn index_mut(&mut self, index: I) -> &mut T::Output {
        self.data.index_mut(index)
    }
}

/// Returns the current .exe module handle.
pub fn get_exe_handle() -> crate::error::Result<winapi::shared::minwindef::HINSTANCE> {
    unsafe {
        let hinstance = winapi::um::libloaderapi::GetModuleHandleW(std::ptr::null());
        if hinstance.is_null() {
            return Err(err!(SystemError("GetModuleHandleW"): Win32Error::last()));
        }
        Ok(hinstance)
    }
}
