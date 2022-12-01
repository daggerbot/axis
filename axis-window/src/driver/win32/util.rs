/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::fmt::{Display, Formatter, Write};
use std::ops::{Deref, DerefMut, Index, IndexMut};

/// Win32 error type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Win32Error(pub u32);

impl Win32Error {
    /// Returns the last Win32 error on the current thread. Returns an error even if the error code
    /// is zero.
    pub fn last() -> Win32Error {
        Win32Error(get_last_error_code())
    }

    /// Returns the last Win32 error on the current thread.
    pub fn try_last() -> Option<Win32Error> {
        match get_last_error_code() {
            0 => None,
            error_code => Some(Win32Error(error_code)),
        }
    }
}

impl Display for Win32Error {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        unsafe {
            let mut buf: *mut u16 = std::ptr::null_mut();
            winapi::um::winbase::FormatMessageW(
                winapi::um::winbase::FORMAT_MESSAGE_ALLOCATE_BUFFER
                    | winapi::um::winbase::FORMAT_MESSAGE_FROM_SYSTEM,
                std::ptr::null(),
                self.0,
                0,
                &mut buf as *mut *mut u16 as *mut u16,
                0,
                std::ptr::null_mut(),
            );

            if buf.is_null() {
                return write!(fmt, "win32 error code {}", self.0);
            }
            let buf = LocalBox::from_raw_parts(buf, crate::util::strlen(buf));
            for char_result in std::char::decode_utf16(buf.iter().copied()) {
                match char_result {
                    Ok(c) => fmt.write_char(c)?,
                    Err(_) => fmt.write_char('?')?,
                }
            }
            Ok(())
        }
    }
}

impl std::error::Error for Win32Error {}

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

/// Returns the last Win32 error code for the current thread.
fn get_last_error_code() -> u32 {
    unsafe { winapi::um::errhandlingapi::GetLastError() }
}
