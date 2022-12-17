/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::ffi::{c_char, CString};
use std::os::unix::io::{AsFd, BorrowedFd};

use math::DivCeil;

use crate::error::Result;
use crate::ffi::CBox;

/// Connection to an X11 server.
pub struct Connection {
    default_screen_index: u8,
    xcb: *mut xcb_sys::xcb_connection_t,
    #[cfg(feature = "x11-sys")]
    xlib: *mut x11_sys::Display,
}

impl Connection {
    /// Returns the index of the default screen.
    pub fn default_screen_index(&self) -> u8 {
        self.default_screen_index
    }

    /// Opens a connection to the specified X server.
    ///
    /// If the `x11-sys` feature is enabled, the Xlib API owns the event queue by default.
    pub fn open<S: Into<Vec<u8>>>(name: S) -> Result<Connection> {
        let c_name = CString::new(name)?;

        unsafe {
            Connection::open_raw(c_name.as_ptr())
        }
    }

    /// Opens a connection to the default X server.
    ///
    /// If the `x11-sys` feature is enabled, the Xlib API owns the event queue by default.
    pub fn open_default() -> Result<Connection> {
        unsafe {
            Connection::open_raw(std::ptr::null())
        }
    }

    /// Returns a pointer to the underlying XCB connection.
    pub fn xcb_connection_ptr(&self) -> *mut xcb_sys::xcb_connection_t {
        self.xcb
    }

    /// Returns a pointer to the underlying Xlib display connection.
    #[cfg(feature = "x11-sys")]
    pub fn xlib_display_ptr(&self) -> *mut x11_sys::Display {
        self.xlib
    }
}

impl Connection {
    /// Thin wrapper around `xcb_change_property()`.
    pub(crate) fn change_property<T: PropertyValue>(&self, mode: ChangePropertyMode, window: u32,
                                                    property: u32, ty: u32, value: &[T])
                                                    -> xcb_sys::xcb_void_cookie_t
    {
        unsafe {
            xcb_sys::xcb_change_property(self.xcb, mode as u8, window, property, ty, T::FORMAT,
                                         u32::try_from(value.len()).unwrap(),
                                         value.as_ptr() as *const _)
        }
    }

    /// Thin wrapper around `xcb_get_property()`.
    pub(crate) fn get_property(&self, delete: bool, window: u32, property: u32, ty: u32,
                               long_offset: u32, long_len: u32)
                               -> xcb_sys::xcb_get_property_cookie_t
    {
        unsafe {
            xcb_sys::xcb_get_property(self.xcb, delete as u8, window, property, ty,
                                      long_offset, long_len)
        }
    }

    /// Sends an X_GetProperty request and immediately gets the reply.
    pub(crate) fn get_property_now(&self, delete: bool, window: u32, property: u32, ty: u32,
                                   long_offset: u32, long_len: u32)
                                   -> Result<GetPropertyReply>
    {
        self.get_property_reply(self.get_property(delete, window, property, ty,
                                                  long_offset, long_len))
    }

    /// Wrapper around `xcb_get_property_reply()` with error handling.
    pub(crate) fn get_property_reply(&self, cookie: xcb_sys::xcb_get_property_cookie_t)
        -> Result<GetPropertyReply>
    {
        unsafe {
            let mut err_ptr = std::ptr::null_mut();
            let reply_ptr = xcb_sys::xcb_get_property_reply(self.xcb, cookie, &mut err_ptr);

            if !err_ptr.is_null() {
                if !reply_ptr.is_null() {
                    libc::free(reply_ptr as *mut _);
                }
                let err = CBox::from_raw(err_ptr);
                return Err(err!(RequestFailed{"X_GetProperty: {:?}", err}));
            } else if reply_ptr.is_null() {
                return Err(err!(RequestFailed("X_GetProperty")));
            }

            Ok(GetPropertyReply {
                reply: CBox::from_raw(reply_ptr),
            })
        }
    }

    /// Gets the entire value of a variable-length window property. Attempts to do this using no
    /// more than two requests, but will use more if the property changes from another thread or
    /// client before this function finishes.
    pub(crate) fn get_property_vec<T: PropertyValue>(&self, window: u32, property: u32, ty: u32)
        -> Result<Option<Vec<T>>>
    {
        let mut long_len = 0;

        loop {
            let reply = self.get_property_now(false, window, property, ty, 0, long_len)?;
            if reply.is_null() {
                return Ok(None);
            } else if reply.reply.bytes_after == 0 {
                return Ok(Some(Vec::from(reply.try_as_slice()?)));
            } else {
                long_len = DivCeil::div_ceil(reply.reply.bytes_after, 4);
            }
        }
    }

    /// Thin wrapper around `xcb_intern_atom()`.
    pub(crate) fn intern_atom<S: ?Sized + AsRef<[u8]>>(&self, name: &S)
        -> xcb_sys::xcb_intern_atom_cookie_t
    {
        let name = name.as_ref();

        unsafe {
            xcb_sys::xcb_intern_atom(self.xcb, 0, u16::try_from(name.len()).unwrap(),
                                     name.as_ptr() as *const c_char)
        }
    }

    /// Wrapper around `xcb_intern_atom_reply()` with error handling.
    pub(crate) fn intern_atom_reply(&self, cookie: xcb_sys::xcb_intern_atom_cookie_t)
        -> Result<u32>
    {
        unsafe {
            let mut err_ptr = std::ptr::null_mut();
            let reply_ptr = xcb_sys::xcb_intern_atom_reply(self.xcb, cookie, &mut err_ptr);
            if reply_ptr.is_null() {
                if err_ptr.is_null() {
                    return Err(err!(RequestFailed("XInternAtom")));
                } else {
                    let err = CBox::from_raw(err_ptr);
                    return Err(err!(RequestFailed{"XInternAtom: {:?}", *err}));
                }
            }
            let reply = CBox::from_raw(reply_ptr);
            Ok(reply.atom)
        }
    }
}

impl Connection {
    /// Opens a connection to the specified X server.
    #[cfg(not(feature = "x11-sys"))]
    unsafe fn open_raw(name: *const c_char) -> Result<Connection> {
        let mut default_screen_index = 0;
        let xcb = xcb_sys::xcb_connect(name, &mut default_screen_index);
        if xcb.is_null() {
            return Err(err!(ConnectionFailed("xcb_connect failed")));
        }

        Ok(Connection {
            default_screen_index: u8::try_from(default_screen_index)?,
            xcb,
        })
    }

    /// Opens a connection to the specified X server.
    #[cfg(feature = "x11-sys")]
    unsafe fn open_raw(name: *const c_char) -> Result<Connection> {
        let xlib = x11_sys::XOpenDisplay(name);
        if xlib.is_null() {
            return Err(err!(ConnectionFailed("XOpenDisplay failed")));
        }

        Ok(Connection {
            default_screen_index: u8::try_from(x11_sys::XDefaultScreen(xlib))?,
            xcb: x11_sys::XGetXCBConnection(xlib) as *mut xcb_sys::xcb_connection_t,
            xlib,
        })
    }
}

impl AsFd for Connection {
    fn as_fd(&self) -> BorrowedFd {
        unsafe { BorrowedFd::borrow_raw(xcb_sys::xcb_get_file_descriptor(self.xcb)) }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        #[cfg(not(feature = "x11-sys"))]
        unsafe {
            xcb_sys::xcb_disconnect(self.xcb);
        }

        #[cfg(feature = "x11-sys")]
        unsafe {
            x11_sys::XCloseDisplay(self.xlib);
        }
    }
}

impl Eq for Connection {}

impl PartialEq for Connection {
    fn eq(&self, rhs: &Connection) -> bool {
        self as *const Connection == rhs as *const Connection
    }
}

/// Determines how a property is changed.
#[allow(dead_code)]
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum ChangePropertyMode {
    Replace = 0,
    Prepend = 1,
    Append = 2,
}

/// Safe wrapper around `xcb_sys::xcb_get_property_reply_t`.
pub struct GetPropertyReply {
    reply: CBox<xcb_sys::xcb_get_property_reply_t>,
}

impl GetPropertyReply {
    /// Returns true if the reply indicates a format of 0, which probably indicates that the
    /// property does not exist.
    pub fn is_null(&self) -> bool {
        self.reply.format == 0
    }

    /// Returns the length of the property in units determined by the format.
    pub fn len(&self) -> usize {
        usize::try_from(self.reply.length).unwrap()
    }

    /// Attempts to get the property value as a slice. Fails if `T` is not the correct type.
    pub fn try_as_slice<T: PropertyValue>(&self) -> Result<&[T]> {
        if self.reply.format != T::FORMAT {
            return Err(err!(EncodingError("x window property format mismatch")));
        }

        unsafe {
            Ok(std::slice::from_raw_parts(
                xcb_sys::xcb_get_property_value((&*self.reply) as *const _) as *const T,
                self.len()))
        }
    }
}

/// Trait for window property values. The property format determines which values are allowed. This
/// is implemented only for signed and unsigned integers of the indicated size.
pub trait PropertyValue: Copy {
    const FORMAT: u8;
}

impl PropertyValue for i8 {
    const FORMAT: u8 = 8;
}

impl PropertyValue for i16 {
    const FORMAT: u8 = 16;
}

impl PropertyValue for i32 {
    const FORMAT: u8 = 32;
}

impl PropertyValue for u8 {
    const FORMAT: u8 = 8;
}

impl PropertyValue for u16 {
    const FORMAT: u8 = 16;
}

impl PropertyValue for u32 {
    const FORMAT: u8 = 32;
}
