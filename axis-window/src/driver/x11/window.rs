/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::os::raw::c_void;
use std::rc::Rc;

use vectorial::Vec2;

use crate::driver::x11::client::{Atoms, Client, Connection, Screen};
use crate::driver::x11::pixel_format::PixelFormat;
use crate::error::Result;
use crate::event::Event;
use crate::window::{IWindow, IWindowBuilder};
use crate::Coord;

/// X11 window builder.
pub struct WindowBuilder<W: 'static + Clone> {
    atoms: Rc<Atoms>,
    connection: Rc<Connection>,
    manager: Rc<WindowManager<W>>,
    _phantom: PhantomData<W>,
    pixel_format: Option<PixelFormat>,
    pos: Option<Vec2<Coord>>,
    screen_num: Option<u8>,
    screens: Rc<Vec<Screen>>,
    size: Option<Vec2<Coord>>,
}

impl<W: 'static + Clone> WindowBuilder<W> {
    /// Gets the underlying connection.
    pub fn connection(&self) -> &Rc<Connection> {
        &self.connection
    }

    /// Gets the screen number on which to build the window.
    pub fn screen_num(&self) -> u8 {
        if let Some(screen_num) = self.screen_num {
            screen_num
        } else {
            self.connection.default_screen_num()
        }
    }

    /// Sets the screen number on which to build the window.
    pub fn with_screen_num(&mut self, screen_num: u8) -> &mut WindowBuilder<W> {
        self.screen_num = Some(screen_num);
        self
    }
}

impl<W: 'static + Clone> WindowBuilder<W> {
    /// Constructs a window builder.
    pub(crate) fn new(client: &Client<W>) -> WindowBuilder<W> {
        WindowBuilder {
            atoms: client.atoms().clone(),
            connection: client.connection().clone(),
            manager: client.window_manager().clone(),
            _phantom: PhantomData,
            pixel_format: None,
            pos: None,
            screen_num: None,
            screens: client.screens_ref().clone(),
            size: None,
        }
    }
}

impl<W: 'static + Clone> IWindowBuilder for WindowBuilder<W> {
    type Client = Client<W>;

    fn build(&self, id: W) -> Result<Window<W>> {
        let window = Window::new(self, id)?;
        window.init_wm_protocols()?;
        Ok(window)
    }
}

/// Data shared between a [Window] and a [WindowManager].
pub struct WindowData<W: 'static + Clone> {
    id: W,
    visible: Cell<bool>,
    xid: Cell<Option<u32>>,
}

impl<W: 'static + Clone> WindowData<W> {
    pub fn id(&self) -> &W {
        &self.id
    }

    pub fn try_xid(&self) -> Result<u32> {
        match self.xid.get() {
            None => Err(err!(ResourceExpired("window expired"))),
            Some(xid) => Ok(xid),
        }
    }

    pub fn update_visibility(&self, visible: bool) -> Option<Event<W>> {
        if self.visible.replace(visible) == visible {
            None
        } else {
            Some(Event::VisibilityChange {
                window_id: self.id.clone(),
                visible: visible,
            })
        }
    }
}

impl<W: 'static + Clone> WindowData<W> {
    fn new(id: W, xid: u32) -> WindowData<W> {
        WindowData {
            id,
            visible: Cell::new(false),
            xid: Cell::new(Some(xid)),
        }
    }
}

/// Manages a map of X11 resource IDs to [WindowData] objects.
pub struct WindowManager<W: 'static + Clone> {
    map: RefCell<HashMap<u32, Rc<WindowData<W>>>>,
}

impl<W: 'static + Clone> WindowManager<W> {
    /// Gets the window with the specified X11 resource ID.
    pub fn get(&self, xid: u32) -> Option<Rc<WindowData<W>>> {
        self.map.borrow().get(&xid).cloned()
    }

    /// Constructs a window manager.
    pub fn new() -> WindowManager<W> {
        WindowManager {
            map: RefCell::new(HashMap::new()),
        }
    }

    /// Registers a resource ID.
    pub fn register(&self, data: Rc<WindowData<W>>) {
        self.map.borrow_mut().insert(data.try_xid().unwrap(), data);
    }

    /// Removes the window with the specified X11 resource ID.
    pub fn unregister(&self, xid: u32) -> Option<Rc<WindowData<W>>> {
        let data = self.map.borrow_mut().remove(&xid);
        if let Some(ref data) = data {
            data.xid.set(None);
        }
        data
    }
}

/// X11 window type.
pub struct Window<W: 'static + Clone> {
    atoms: Rc<Atoms>,
    connection: Rc<Connection>,
    data: Rc<WindowData<W>>,
    xcb: *mut xcb_sys::xcb_connection_t,
}

impl<W: 'static + Clone> Window<W> {
    /// Returns the underlying connection.
    pub fn connection(&self) -> &Rc<Connection> {
        &self.connection
    }

    /// Returns the X11 resource ID for the window, or an error if the window has expired.
    pub fn try_xid(&self) -> Result<u32> {
        self.data.try_xid()
    }

    /// Returns the X11 resource ID for the window, or `None` if the window has expired.
    pub fn xid(&self) -> Option<u32> {
        self.data.xid.get()
    }
}

impl<W: 'static + Clone> Window<W> {
    fn init_wm_protocols(&self) -> Result<()> {
        self.set_wm_protocols([
            self.atoms.WM_DELETE_WINDOW,
        ].as_ref())
    }

    fn set_property<T: ?Sized + PropertyData>(&self, property: u32, ty: u32, data: &T)
        -> Result<xcb_sys::xcb_void_cookie_t>
    {
        Ok(self.connection.change_property(ChangePropertyMode::Replace, self.try_xid()?, property,
                                           ty, data))
    }

    fn set_wm_protocols(&self, protocols: &[u32]) -> Result<()> {
        self.set_property(self.atoms.WM_PROTOCOLS, xcb_sys::XCB_ATOM_ATOM, protocols)?;
        Ok(())
    }

    /// Creates and registers a new window with `xcb_create_window()` but does not do any other
    /// initialization.
    fn new(builder: &WindowBuilder<W>, id: W) -> Result<Window<W>> {
        let connection = builder.connection.clone();
        let xcb = connection.xcb_connection_ptr();
        let screen_num = match builder.screen_num {
            None => builder.connection.default_screen_num(),
            Some(n) => {
                if usize::from(n) >= builder.screens.len() {
                    return Err(err!(InvalidArgument("invalid X11 screen number")));
                }
                n
            },
        };
        let pixel_format = match builder.pixel_format {
            None => builder.screens[screen_num as usize].default_pixel_format(),
            Some(ref pixel_format) => {
                if *pixel_format.connection() != builder.connection
                   || pixel_format.screen_num() != screen_num
                {
                    return Err(err!(IncompatibleResource("incompatible pixel format")));
                }
                pixel_format.clone()
            },
        };
        let depth = pixel_format.depth();
        let xid;
        let parent = builder.screens[screen_num as usize].root();
        let pos = match builder.pos {
            None => Vec2::new(0, 0),
            Some(pos) => Vec2::new(clamp_pos(pos.x), clamp_pos(pos.y)),
        };
        let size = match builder.size {
            None => Vec2::new(100, 100),
            Some(size) => Vec2::new(clamp_size(size.x), clamp_size(size.y)),
        };
        let visual_id = pixel_format.visual_id();
        let values = vec! {
            xcb_sys::XCB_EVENT_MASK_STRUCTURE_NOTIFY as u32,
        };
        let value_mask = xcb_sys::XCB_CW_EVENT_MASK;

        unsafe {
            xid = xcb_sys::xcb_generate_id(xcb);
            xcb_sys::xcb_create_window(xcb, depth, xid, parent, pos.x, pos.y, size.x, size.y, 0,
                                       xcb_sys::XCB_WINDOW_CLASS_INPUT_OUTPUT as u16, visual_id,
                                       value_mask, values.as_ptr() as *const _);
        }

        let data = Rc::new(WindowData::new(id, xid));
        builder.manager.register(data.clone());

        Ok(Window {
            atoms: builder.atoms.clone(),
            connection,
            data,
            xcb,
        })
    }
}

impl<W: 'static + Clone> Drop for Window<W> {
    fn drop(&mut self) {
        self.destroy();
    }
}

impl<W: 'static + Clone> IWindow for Window<W> {
    type Client = Client<W>;

    fn destroy(&self) {
        if let Some(xid) = self.data.xid.take() {
            unsafe {
                xcb_sys::xcb_destroy_window(self.xcb, xid);
            }
        }
    }

    fn id(&self) -> &W {
        &self.data.id
    }

    fn is_visible(&self) -> bool {
        self.xid().is_some() && self.data.visible.get()
    }

    fn set_visible(&self, visible: bool) -> Result<()> {
        unsafe {
            if visible {
                xcb_sys::xcb_map_window(self.xcb, self.try_xid()?);
            } else if let Some(xid) = self.xid() {
                xcb_sys::xcb_unmap_window(self.xcb, xid);
            }
        }

        Ok(())
    }
}

/// Modes for property change requests.
#[allow(dead_code)]
#[derive(Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum ChangePropertyMode {
    Replace = 0,
    Prepend = 1,
    Append = 2,
}

/// Trait for window property values.
pub trait PropertyData {
    fn as_ptr(&self) -> *const c_void;
    fn format() -> u8;
    fn len(&self) -> u32;
}

impl PropertyData for [u32] {
    fn as_ptr(&self) -> *const c_void {
        self.as_ptr() as *const c_void
    }

    fn format() -> u8 { 32 }

    fn len(&self) -> u32 {
        self.len() as u32
    }
}

/// Clamps a positional coordinate within acceptable values.
fn clamp_pos(n: Coord) -> i16 {
    if n < Coord::from(i16::MIN) {
        i16::MIN
    } else if n > Coord::from(i16::MAX) {
        i16::MAX
    } else {
        n as i16
    }
}

/// Clamps a size value within acceptable values.
fn clamp_size(n: Coord) -> u16 {
    if n < 1 {
        1
    } else if n > Coord::from(u16::MAX) {
        u16::MAX
    } else {
        n as u16
    }
}
