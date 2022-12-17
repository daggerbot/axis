/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::fmt::{Display, Formatter, Write};

use crate::driver::win32::ffi::LocalBox;

/// Win32 error type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Win32Error {
    error_code: u32,
}

impl Win32Error {
    /// Returns the error code.
    pub fn code(&self) -> u32 {
        self.error_code
    }

    /// Returns the last Win32 error on the current thread. Returns an error even if the error code
    /// is zero.
    pub fn last() -> Win32Error {
        Win32Error {
            error_code: get_last_error_code(),
        }
    }

    /// Returns the last Win32 error on the current thread.
    pub fn try_last() -> Option<Win32Error> {
        match get_last_error_code() {
            0 => None,
            error_code => Some(Win32Error { error_code }),
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
                std::ptr::null(), self.error_code, 0, &mut buf as *mut *mut u16 as *mut u16, 0,
                std::ptr::null_mut());

            if buf.is_null() {
                return write!(fmt, "win32 error code {}", self.error_code);
            }
            let buf = LocalBox::from_raw_parts(buf, crate::ffi::strlen(buf));
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

/// Sets the current thread's error code to 0.
pub fn clear_last_error() {
    unsafe {
        winapi::um::errhandlingapi::SetLastError(0);
    }
}

/// Returns the last Win32 error code for the current thread.
pub fn get_last_error_code() -> u32 {
    unsafe {
        winapi::um::errhandlingapi::GetLastError()
    }
}
