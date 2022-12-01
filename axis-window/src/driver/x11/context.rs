/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::cell::RefCell;
use std::ffi::{c_char, CString};
use std::marker::PhantomData;
use std::os::unix::io::{AsFd, BorrowedFd};
use std::rc::Rc;

use crate::context::IContext;
use crate::driver::x11::device::{Device, Devices};
use crate::driver::x11::pixel_format::{PixelFormat, PixelFormats};
use crate::driver::x11::window::{Window, WindowBuilder, WindowManager};
use crate::error::Result;

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

        unsafe { Connection::open_raw(c_name.as_ptr()) }
    }

    /// Opens a connection to the default X server.
    ///
    /// If the `x11-sys` feature is enabled, the Xlib API owns the event queue by default.
    pub fn open_default() -> Result<Connection> {
        unsafe { Connection::open_raw(std::ptr::null()) }
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

/// X11 window system context.
pub struct Context<W: 'static + Clone> {
    connection: Rc<Connection>,
    _phantom: PhantomData<W>,
    window_manager: Rc<RefCell<WindowManager<W>>>,
}

impl<W: 'static + Clone> Context<W> {
    /// Returns the underlying X connection.
    pub fn connection(&self) -> &Rc<Connection> {
        &self.connection
    }

    /// Opens a connection to the specified X server.
    pub fn open<S: Into<Vec<u8>>>(name: S) -> Result<Context<W>> {
        Context::init(Connection::open(name)?)
    }

    /// Opens a connectoin to the default X server.
    pub fn open_default() -> Result<Context<W>> {
        Context::init(Connection::open_default()?)
    }
}

impl<W: 'static + Clone> Context<W> {
    pub(crate) fn window_manager(&self) -> &Rc<RefCell<WindowManager<W>>> {
        &self.window_manager
    }
}

impl<W: 'static + Clone> Context<W> {
    /// Performs any initialization on the context that occurs after the connection is obtained.
    fn init(connection: Connection) -> Result<Context<W>> {
        #[cfg(feature = "x11-sys")]
        unsafe {
            x11_sys::XSetEventQueueOwner(
                connection.xlib,
                x11_sys::XEventQueueOwner_XCBOwnsEventQueue,
            );
        }

        Ok(Context {
            connection: Rc::new(connection),
            _phantom: PhantomData,
            window_manager: Rc::new(RefCell::new(WindowManager::new())),
        })
    }
}

impl<W: 'static + Clone> Eq for Context<W> {}

impl<W: 'static + Clone> IContext for Context<W> {
    type Device = Device<W>;
    type Devices = Devices<W>;
    type PixelFormat = PixelFormat;
    type PixelFormats = PixelFormats;
    type Window = Window<W>;
    type WindowBuilder = WindowBuilder<W>;
    type WindowId = W;

    fn default_device(&self) -> Device<W> {
        for device in self.devices() {
            if device.screen_index() == self.connection.default_screen_index {
                return device;
            }
        }
        panic!("invalid default X screen");
    }

    fn devices(&self) -> Devices<W> {
        Devices::new(&self)
    }
}

impl<W: 'static + Clone> PartialEq for Context<W> {
    fn eq(&self, rhs: &Context<W>) -> bool {
        self.connection == rhs.connection
    }
}
