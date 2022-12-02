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

use math::{TryFromComposite, Vector2};

use crate::device::IDevice;
use crate::driver::x11::context::{Atoms, Connection, Context};
use crate::driver::x11::device::Device;
use crate::driver::x11::pixel_format::PixelFormat;
use crate::error::Result;
use crate::util::{clamp, CBox};
use crate::window::{IWindow, IWindowBuilder, WindowPos};
use crate::Coord;

const EXPIRED_MSG: &'static str = "window destroyed";

/// Parameters for creating an X11 window.
pub struct WindowBuilder<W: 'static + Clone> {
    device: Device<W>,
    pixel_format: Option<PixelFormat>,
    pos: WindowPos,
    size: Option<Vector2<Coord>>,
    title: Option<String>,
    visible: bool,
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
            title: None,
            visible: false,
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

        let mut attrs = Vec::new();
        let attr_mask = xcb_sys::XCB_CW_EVENT_MASK;
        attrs.push(xcb_sys::XCB_EVENT_MASK_STRUCTURE_NOTIFY);

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
                attr_mask,
                attrs.as_ptr() as *const _,
            );
        }

        let shared = Rc::new(WindowShared {
            id,
            parent_xid: Cell::new(Some(self.device.root_window_id())),
            pos: Cell::new(Some(pos)),
            root_xid: self.device.root_window_id(),
            size: Cell::new(Some(size)),
            visible: Cell::new(false),
            xid: Cell::new(Some(xid)),
        });

        manager.borrow_mut().map.insert(xid, shared.clone());

        let mut window = Window {
            atoms: self.device.atoms().clone(),
            connection,
            shared,
            xcb,
        };

        if let Some(ref title) = self.title {
            window.set_title(title.as_str())?;
        }
        if self.visible {
            window.set_visible(true)?;
        }

        // Subscribe to close events.
        window.set_atom_property(atoms.wm_protocols, &[atoms.wm_delete_window])?;

        Ok(window)
    }

    fn with_pos(&mut self, pos: WindowPos) -> &mut WindowBuilder<W> {
        self.pos = pos;
        self
    }

    fn with_size(&mut self, size: Option<Vector2<Coord>>) -> &mut WindowBuilder<W> {
        self.size = size;
        self
    }

    fn with_title<S: Into<String>>(&mut self, title: S) -> &mut WindowBuilder<W> {
        self.title = Some(title.into());
        self
    }

    fn with_visibility(&mut self, visible: bool) -> &mut WindowBuilder<W> {
        self.visible = visible;
        self
    }
}

/// Data shared between a [`Window`] and a [`WindowManager`].
pub struct WindowShared<W: 'static + Clone> {
    id: W,
    parent_xid: Cell<Option<u32>>,
    pos: Cell<Option<Vector2<i16>>>,
    root_xid: u32,
    size: Cell<Option<Vector2<u16>>>,
    visible: Cell<bool>,
    xid: Cell<Option<u32>>,
}

impl<W: 'static + Clone> WindowShared<W> {
    pub fn id(&self) -> &W {
        &self.id
    }

    pub fn is_parent_root(&self) -> bool {
        self.parent_xid.get() == Some(self.root_xid)
    }

    pub fn try_xid(&self) -> Result<u32> {
        match self.xid.get() {
            None => Err(err!(ResourceExpired(EXPIRED_MSG))),
            Some(xid) => Ok(xid),
        }
    }

    pub fn update_parent_xid(&self, parent_xid: u32) {
        self.parent_xid.set(Some(parent_xid));
    }

    pub fn update_pos(&self, pos: Vector2<i16>) -> bool {
        self.pos.replace(Some(pos)) != Some(pos)
    }

    pub fn update_size(&self, size: Vector2<u16>) -> bool {
        self.size.replace(Some(size)) != Some(size)
    }

    pub fn update_visibility(&self, visible: bool) -> bool {
        self.visible.replace(visible) != visible
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
                window.parent_xid.set(None);
                window.pos.set(None);
                window.size.set(None);
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
    atoms: Rc<Atoms>,
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
    pub(crate) fn get_u8_array_property(&self, property: u32, ty: u32) -> Result<Option<Vec<u8>>> {
        const LONG_LENGTH: u16 = 64;
        let xid = self.try_xid()?;
        let mut long_offset = 0;
        let mut value = Vec::new();

        'request_loop: loop {
            unsafe {
                let cookie = xcb_sys::xcb_get_property(
                    self.xcb,
                    0,
                    xid,
                    property,
                    ty,
                    long_offset,
                    u32::from(LONG_LENGTH),
                );
                let mut err_ptr = std::ptr::null_mut();
                let reply_ptr = xcb_sys::xcb_get_property_reply(self.xcb, cookie, &mut err_ptr);
                if reply_ptr.is_null() {
                    if err_ptr.is_null() {
                        return Err(err!(RequestFailed("X_GetProperty")));
                    } else {
                        let err = CBox::from_raw(err_ptr);
                        return Err(err!(RequestFailed{"X_GetProperty: {:?}", err}));
                    }
                }
                let reply = CBox::from_raw(reply_ptr);
                let value_len =
                    usize::try_from(xcb_sys::xcb_get_property_value_length(reply.as_ptr()))?;

                if reply.format == 0 && value.is_empty() {
                    return Ok(None);
                } else if reply.format != 8 {
                    println!("{:?}", reply);
                    return Err(err!(EncodingError("x property format mismatch")));
                }

                value.extend(std::slice::from_raw_parts(
                    xcb_sys::xcb_get_property_value(reply.as_ptr()) as *const u8,
                    value_len,
                ));

                if value_len < usize::from(LONG_LENGTH) * 4 {
                    break 'request_loop;
                } else {
                    long_offset += u32::from(LONG_LENGTH);
                }
            }
        }

        Ok(Some(value))
    }

    pub(crate) fn set_atom_property(&self, property: u32, value: &[u32]) -> Result<()> {
        self.set_u32_property(property, xcb_sys::XCB_ATOM_ATOM, value)
    }

    pub(crate) fn set_u8_property(&self, property: u32, ty: u32, value: &[u8]) -> Result<()> {
        let xid = self.try_xid()?;

        unsafe {
            xcb_sys::xcb_change_property(
                self.xcb,
                xcb_sys::XCB_PROP_MODE_REPLACE as u8,
                xid,
                property,
                ty,
                8,
                u32::try_from(value.len()).unwrap(),
                value.as_ptr() as *const _,
            );
        }

        Ok(())
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

    fn destroy(&mut self) {
        if let Some(xid) = self.shared.xid.take() {
            unsafe {
                xcb_sys::xcb_destroy_window(self.xcb, xid);
            }
        }
    }

    fn id(&self) -> &W {
        &self.shared.id
    }

    fn is_alive(&self) -> bool {
        self.shared.xid.get().is_some()
    }

    fn is_visible(&self) -> bool {
        self.shared.visible.get()
    }

    fn pos(&self) -> Result<Vector2<Coord>> {
        match self.shared.pos.get() {
            None => Err(err!(ResourceExpired(EXPIRED_MSG))),
            Some(pos) => Ok(Vector2::try_from_composite(pos)?),
        }
    }

    fn set_pos(&mut self, pos: Vector2<Coord>) -> Result<()> {
        let xid = self.try_xid()?;
        let x = clamp(pos.x, Coord::from(i16::MIN), Coord::from(i16::MAX)) as i16;
        let y = clamp(pos.y, Coord::from(i16::MIN), Coord::from(i16::MAX)) as i16;

        unsafe {
            xcb_sys::xcb_configure_window(
                self.xcb,
                xid,
                (xcb_sys::XCB_CONFIG_WINDOW_X | xcb_sys::XCB_CONFIG_WINDOW_Y) as u16,
                [x as u32, y as u32].as_ptr() as *const _,
            );
        }

        Ok(())
    }

    fn set_size(&mut self, size: Vector2<Coord>) -> Result<()> {
        let xid = self.try_xid()?;
        let width = clamp(size.x, 1, Coord::from(u16::MAX)) as u16;
        let height = clamp(size.y, 1, Coord::from(u16::MAX)) as u16;

        unsafe {
            xcb_sys::xcb_configure_window(
                self.xcb,
                xid,
                (xcb_sys::XCB_CONFIG_WINDOW_WIDTH | xcb_sys::XCB_CONFIG_WINDOW_HEIGHT) as u16,
                [width as u32, height as u32].as_ptr() as *const _,
            );
        }

        Ok(())
    }

    fn set_title(&mut self, title: &str) -> Result<()> {
        let bytes = title.as_bytes();
        self.set_u8_property(xcb_sys::XCB_ATOM_WM_NAME, xcb_sys::XCB_ATOM_STRING, bytes)?;
        self.set_u8_property(
            xcb_sys::XCB_ATOM_WM_ICON_NAME,
            xcb_sys::XCB_ATOM_STRING,
            bytes,
        )?;
        self.set_u8_property(self.atoms.net_wm_name, self.atoms.utf8_string, bytes)?;
        self.set_u8_property(self.atoms.net_wm_icon_name, self.atoms.utf8_string, bytes)?;
        Ok(())
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

    fn size(&self) -> Result<Vector2<Coord>> {
        match self.shared.size.get() {
            None => Err(err!(ResourceExpired(EXPIRED_MSG))),
            Some(size) => Ok(Vector2::try_from_composite(size)?),
        }
    }

    fn title(&self) -> Result<String> {
        match self.get_u8_array_property(self.atoms.net_wm_name, self.atoms.utf8_string)? {
            None => (),
            Some(bytes) => return Ok(String::from_utf8(bytes)?),
        }
        match self.get_u8_array_property(xcb_sys::XCB_ATOM_WM_NAME, xcb_sys::XCB_ATOM_STRING)? {
            None => Ok(String::new()),
            Some(bytes) => Ok(String::from_utf8(bytes)?),
        }
    }
}
