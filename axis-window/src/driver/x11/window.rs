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

use crate::driver::x11::context::{Connection, Context};
use crate::driver::x11::device::Device;
use crate::driver::x11::pixel_format::PixelFormat;
use crate::error::Result;
use crate::util::clamp;
use crate::window::{IWindow, IWindowBuilder, WindowPos};
use crate::IDevice;

/// Parameters for creating an X11 window.
pub struct WindowBuilder<W: 'static + Clone> {
    device: Device<W>,
    pixel_format: Option<PixelFormat>,
    pos: WindowPos,
    size: Option<Vector2<usize>>,
}

impl<W: 'static + Clone> WindowBuilder<W> {
    /// Returns the underlying X connection.
    pub fn connection(&self) -> &Rc<Connection> {
        self.device.connection()
    }

    /// Returns the device on which the window is to be created.
    pub fn device(&self) -> &Device<W> {
        &self.device
    }
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
        let atoms = self.device.atoms().clone();
        let manager = self.device.window_manager().clone();
        let pixel_format = self
            .pixel_format
            .clone()
            .unwrap_or_else(|| self.device.default_pixel_format());
        let pos = match self.pos {
            WindowPos::Default => Vector2::new(0, 0),  // TODO
            WindowPos::Centered => Vector2::new(0, 0), // TODO
            WindowPos::Point(pos) => Vector2 {
                x: clamp(pos.x, -0x8000, 0x7fff) as i16,
                y: clamp(pos.y, -0x8000, 0x7fff) as i16,
            },
        };
        let size = match self.size {
            None => Vector2::new(640, 480), // TODO
            Some(size) => Vector2 {
                x: clamp(size.x, 1, 0xffff) as u16,
                y: clamp(size.y, 1, 0xffff) as u16,
            },
        };
        let xid;

        unsafe {
            xid = xcb_sys::xcb_generate_id(xcb);
            xcb_sys::xcb_create_window(
                xcb,
                pixel_format.depth(),
                xid,
                self.device.root_window_id(),
                pos.x,
                pos.y,
                size.x,
                size.y,
                0,
                xcb_sys::XCB_WINDOW_CLASS_INPUT_OUTPUT as u16,
                pixel_format.visual_id(),
                0,
                std::ptr::null(),
            );
        }

        let shared = Rc::new(WindowShared {
            id,
            visible: Cell::new(false),
            xid: Cell::new(Some(xid)),
        });

        manager.borrow_mut().map.insert(xid, shared.clone());

        let window = Window {
            connection,
            shared,
            xcb,
        };

        // Subscribe to close events.
        window.set_atom_property(atoms.wm_protocols, &[atoms.wm_delete_window])?;

        Ok(window)
    }
}

/// Data shared between a [`Window`] and a [`WindowManager`].
pub struct WindowShared<W: 'static + Clone> {
    id: W,
    visible: Cell<bool>,
    xid: Cell<Option<u32>>,
}

impl<W: 'static + Clone> WindowShared<W> {
    pub fn id(&self) -> &W {
        &self.id
    }

    pub fn try_xid(&self) -> Result<u32> {
        match self.xid.get() {
            None => Err(err!(ResourceExpired("window destroyed"))),
            Some(xid) => Ok(xid),
        }
    }

    pub fn update_visibility(&self, visible: bool) {
        self.visible.set(visible);
    }
}

/// Window ID map.
pub struct WindowManager<W: 'static + Clone> {
    map: HashMap<u32, Rc<WindowShared<W>>>,
}

impl<W: 'static + Clone> WindowManager<W> {
    /// Removes a window from the manager and sets its X ID to `None`, thus marking it as destroyed.
    pub fn expire(&mut self, xid: u32) -> Option<Rc<WindowShared<W>>> {
        match self.map.remove(&xid) {
            None => None,
            Some(window) => {
                window.xid.set(None);
                Some(window)
            },
        }
    }

    /// Gets a window from its X ID.
    pub fn get(&self, xid: u32) -> Option<&Rc<WindowShared<W>>> {
        self.map.get(&xid)
    }

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
    pub fn connection(&self) -> &Rc<Connection> {
        &self.connection
    }

    /// Returns the underlying X ID, or an error if the window has expired.
    pub fn try_xid(&self) -> Result<u32> {
        self.shared.try_xid()
    }
}

impl<W: 'static + Clone> Window<W> {
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

    pub(crate) fn set_atom_property(&self, property: u32, value: &[u32]) -> Result<()> {
        self.set_u32_property(property, xcb_sys::XCB_ATOM_ATOM, value)
    }

    pub(crate) fn set_u32_property(&self, property: u32, ty: u32, value: &[u32]) -> Result<()> {
        let xid = self.try_xid()?;

        unsafe {
            xcb_sys::xcb_change_property(
                self.xcb,
                xcb_sys::XCB_PROP_MODE_REPLACE as u8,
                xid,
                property,
                ty,
                32,
                u32::try_from(value.len()).unwrap(),
                value.as_ptr() as *const _,
            );
        }

        Ok(())
    }
}

impl<W: 'static + Clone> Drop for Window<W> {
    fn drop(&mut self) {
        self.destroy();
    }
}

impl<W: 'static + Clone> IWindow for Window<W> {
    type Context = Context<W>;

    fn id(&self) -> &W {
        &self.shared.id
    }

    fn is_alive(&self) -> bool {
        self.shared.xid.get().is_some()
    }

    fn is_visible(&self) -> bool {
        self.shared.visible.get()
    }

    fn set_visible(&mut self, visible: bool) -> Result<()> {
        let xid = self.try_xid()?;

        unsafe {
            if visible {
                xcb_sys::xcb_map_window(self.xcb, xid);
            } else {
                xcb_sys::xcb_unmap_window(self.xcb, xid);
            }
        }

        Ok(())
    }
}
