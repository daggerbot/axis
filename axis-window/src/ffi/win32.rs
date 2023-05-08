/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::fmt::{Display, Formatter};

use winapi::shared::minwindef::HMODULE;

/// Win32 error type.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Error {
    code: u32,
}

impl Error {
    /// Returns the underlying error code.
    pub fn code(self) -> u32 {
        self.code
    }

    /// Constructs an error from a non-zero error code.
    pub fn from_code(code: u32) -> Option<Error> {
        match code {
            0 => None,
            _ => Some(Error { code }),
        }
    }

    /// Returns the Windows API error on the calling thread.
    pub fn get() -> Option<Error> {
        Error::from_code(Error::get_code())
    }

    /// Returns the Windows API error code on the calling thread.
    pub fn get_code() -> u32 {
        unsafe {
            winapi::um::errhandlingapi::GetLastError()
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let mut buf: *mut u16 = std::ptr::null_mut();
        let buf_ptr = &mut buf as *mut *mut u16;
        let message;

        unsafe {
            winapi::um::winbase::FormatMessageW(
                winapi::um::winbase::FORMAT_MESSAGE_FROM_SYSTEM
                | winapi::um::winbase::FORMAT_MESSAGE_ALLOCATE_BUFFER,
                std::ptr::null(), self.code, 0, buf_ptr as *mut u16, 0, std::ptr::null_mut());

            if buf.is_null() {
                return write!(f, "Win32 error code {}", self.code);
            }

            let len = libc::wcslen(buf) as usize;
            let message16 = std::slice::from_raw_parts(buf, len);
            message = String::from_utf16_lossy(message16);
            winapi::um::winbase::LocalFree(buf as *mut _);
        }

        f.write_str(message.as_str())
    }
}

impl std::error::Error for Error {}

/// Gets the current executable's handle.
pub fn get_exe_handle() -> crate::Result<HMODULE> {
    let handle;

    unsafe {
        handle = winapi::um::libloaderapi::GetModuleHandleW(std::ptr::null());
    }

    if handle.is_null() {
        return Err(err!(RuntimeError("GetModuleHandleW"): ??w));
    }
    Ok(handle)
}
