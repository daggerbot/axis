/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::device::IDevice;
use crate::driver::x11::connection::Connection;
use crate::driver::x11::pixel_format::{PixelFormat, PixelFormats};
use crate::driver::x11::system::{Atoms, System};
use crate::driver::x11::window::{WindowBuilder, WindowManager};

/// X11 window system type which corresponds to an X "screen".
#[derive(Clone)]
pub struct Device<W: 'static + Clone> {
    atoms: Rc<Atoms>,
    connection: Rc<Connection>,
    _phantom_data: PhantomData<W>,
    screen_index: u8,
    screen_ptr: *mut xcb_sys::xcb_screen_t,
    window_manager: Rc<RefCell<WindowManager<W>>>,
}

impl<W: 'static + Clone> Device<W> {
    /// Returns the underlying X connection.
    pub fn connection(&self) -> &Rc<Connection> {
        &self.connection
    }

    /// Returns the XID for the visual used by the root window.
    pub fn root_visual_id(&self) -> u32 {
        unsafe { (*self.screen_ptr).root_visual }
    }

    /// Returns the XID of the root window.
    pub fn root_window_id(&self) -> u32 {
        unsafe { (*self.screen_ptr).root }
    }

    /// Returns the X screen index.
    pub fn screen_index(&self) -> u8 {
        self.screen_index
    }

    /// Returns the underlying XCB screen structure pointer.
    pub fn xcb_screen_ptr(&self) -> *mut xcb_sys::xcb_screen_t {
        self.screen_ptr
    }
}

impl<W: 'static + Clone> Device<W> {
    pub(crate) fn atoms(&self) -> &Rc<Atoms> {
        &self.atoms
    }

    pub(crate) fn window_manager(&self) -> &Rc<RefCell<WindowManager<W>>> {
        &self.window_manager
    }
}

impl<W: 'static + Clone> Eq for Device<W> {}

impl<W: 'static + Clone> IDevice for Device<W> {
    type System = System<W>;

    fn default_pixel_format(&self) -> PixelFormat {
        let visual_id = self.root_visual_id();
        for pixel_format in self.pixel_formats() {
            if pixel_format.visual_id() == visual_id {
                return pixel_format;
            }
        }
        panic!("invalid root visual");
    }

    fn new_window(&self) -> WindowBuilder<W> {
        WindowBuilder::new(self)
    }

    fn pixel_formats(&self) -> PixelFormats {
        PixelFormats::new(self)
    }
}

impl<W: 'static + Clone> PartialEq for Device<W> {
    fn eq(&self, rhs: &Device<W>) -> bool {
        self.connection == rhs.connection && self.screen_index == rhs.screen_index
    }
}

/// Iterator over available X screens.
pub struct Devices<W: 'static + Clone> {
    atoms: Rc<Atoms>,
    connection: Rc<Connection>,
    iter: xcb_sys::xcb_screen_iterator_t,
    _phantom_data: PhantomData<W>,
    screen_index: u8,
    window_manager: Rc<RefCell<WindowManager<W>>>,
}

impl<W: 'static + Clone> Devices<W> {
    pub(crate) fn new(system: &System<W>) -> Devices<W> {
        let connection = system.connection().clone();
        let xcb = connection.xcb_connection_ptr();

        unsafe {
            Devices {
                atoms: system.atoms().clone(),
                connection,
                iter: xcb_sys::xcb_setup_roots_iterator(xcb_sys::xcb_get_setup(xcb)),
                _phantom_data: PhantomData,
                screen_index: 0,
                window_manager: system.window_manager().clone(),
            }
        }
    }
}

impl<W: 'static + Clone> Iterator for Devices<W> {
    type Item = Device<W>;

    fn next(&mut self) -> Option<Device<W>> {
        if self.iter.rem == 0 {
            return None;
        }

        let device = Device {
            atoms: self.atoms.clone(),
            connection: self.connection.clone(),
            _phantom_data: PhantomData,
            screen_index: self.screen_index,
            screen_ptr: self.iter.data,
            window_manager: self.window_manager.clone(),
        };

        unsafe {
            xcb_sys::xcb_screen_next(&mut self.iter);
        }

        self.screen_index += 1;
        Some(device)
    }
}
