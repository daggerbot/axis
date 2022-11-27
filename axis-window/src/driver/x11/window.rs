/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use math::Vector2;

use crate::IDevice;
use crate::driver::x11::context::{Connection, Context};
use crate::driver::x11::device::Device;
use crate::driver::x11::pixel_format::PixelFormat;
use crate::error::Result;
use crate::util::clamp;
use crate::window::{IWindow, IWindowBuilder, WindowPos};

/// Parameters for creating an X11 window.
pub struct WindowBuilder<W: 'static + Clone> {
    device: Device<W>,
    pixel_format: Option<PixelFormat>,
    pos: WindowPos,
    size: Option<Vector2<usize>>,
}

impl<W: 'static + Clone> WindowBuilder<W> {
    /// Returns the underlying X connection.
    pub fn connection(&self) -> &Rc<Connection> { self.device.connection() }

    /// Returns the device on which the window is to be created.
    pub fn device(&self) -> &Device<W> { &self.device }
}

impl<W: 'static + Clone> WindowBuilder<W> {
    pub(crate) fn new(device: &Device<W>) -> WindowBuilder<W> {
        WindowBuilder {
            device: device.clone(),
            pixel_format: None,
            pos: WindowPos::Default,
            size: None,
        }
    }
}

impl<W: 'static + Clone> IWindowBuilder for WindowBuilder<W> {
    type Context = Context<W>;

    fn build(&self, id: W) -> Result<Window<W>> {
        let connection = self.device.connection().clone();
        let xcb = connection.xcb_connection_ptr();
        let manager = self.device.window_manager().clone();
        let pixel_format = self.pixel_format.clone()
                           .unwrap_or_else(|| self.device.default_pixel_format());
        let pos = match self.pos {
            WindowPos::Default => Vector2::new(0, 0), // TODO
            WindowPos::Centered => Vector2::new(0, 0), // TODO
            WindowPos::Point(pos) => {
                Vector2 {
                    x: clamp(pos.x, -0x8000, 0x7fff) as i16,
                    y: clamp(pos.y, -0x8000, 0x7fff) as i16,
                }
            },
        };
        let size = match self.size {
            None => Vector2::new(640, 480), // TODO
            Some(size) => {
                Vector2 {
                    x: clamp(size.x, 1, 0xffff) as u16,
                    y: clamp(size.y, 1, 0xffff) as u16,
                }
            },
        };
        let xid;

        unsafe {
            xid = xcb_sys::xcb_generate_id(xcb);
            xcb_sys::xcb_create_window(xcb, pixel_format.depth(), xid, self.device.root_window_id(),
                                       pos.x, pos.y, size.x, size.y, 0,
                                       xcb_sys::XCB_WINDOW_CLASS_INPUT_OUTPUT as u16,
                                       pixel_format.visual_id(), 0, std::ptr::null());
        }

        let shared = Rc::new(WindowShared {
            id,
            visible: Cell::new(false),
            xid: Cell::new(Some(xid)),
        });

        manager.borrow_mut().map.insert(xid, shared.clone());

        Ok(Window {
            connection,
            shared,
            xcb,
        })
    }
}

/// Data shared between a [`Window`] and a [`WindowManager`].
pub struct WindowShared<W: 'static + Clone> {
    id: W,
    visible: Cell<bool>,
    xid: Cell<Option<u32>>,
}

/// Window ID map.
pub struct WindowManager<W: 'static + Clone> {
    map: HashMap<u32, Rc<WindowShared<W>>>,
}

impl<W: 'static + Clone> WindowManager<W> {
    /// Constructs a new window manager.
    pub fn new() -> WindowManager<W> {
        WindowManager {
            map: HashMap::new(),
        }
    }
}

/// Top-level X11 window type.
pub struct Window<W: 'static + Clone> {
    connection: Rc<Connection>,
    shared: Rc<WindowShared<W>>,
    xcb: *mut xcb_sys::xcb_connection_t,
}

impl<W: 'static + Clone> Window<W> {
    /// Returns the underlying X connection.
    pub fn connection(&self) -> &Rc<Connection> { &self.connection }
}

impl<W: 'static + Clone> Window<W> {
    /// Destroys the window.
    pub(crate) fn destroy(&self) -> bool {
        match self.shared.xid.take() {
            None => false,
            Some(xid) => {
                unsafe {
                    xcb_sys::xcb_destroy_window(self.xcb, xid);
                }
                true
            },
        }
    }
}

impl<W: 'static + Clone> Drop for Window<W> {
    fn drop(&mut self) {
        self.destroy();
    }
}

impl<W: 'static + Clone> IWindow for Window<W> {
    type Context = Context<W>;

    fn id(&self) -> &W { &self.shared.id }
    fn is_alive(&self) -> bool { self.shared.xid.get().is_some() }
    fn is_visible(&self) -> bool { self.shared.visible.get() }
}
