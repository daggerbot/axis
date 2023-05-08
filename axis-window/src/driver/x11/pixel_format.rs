/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::fmt::{Display, Formatter};
use std::rc::Rc;

use crate::driver::x11::client::Connection;
use crate::pixel_format::IPixelFormat;

/// X11 pixel format type.
#[derive(Clone)]
pub struct PixelFormat {
    connection: Rc<Connection>,
    depth: u8,
    screen_num: u8,
    visualtype_ptr: *mut xcb_sys::xcb_visualtype_t,
}

impl PixelFormat {
    /// Gets the number of bits per RGB value.
    pub fn bits_per_rgb_value(&self) -> u8 {
        unsafe {
            (*self.visualtype_ptr).bits_per_rgb_value
        }
    }

    /// Gets the blue mask.
    pub fn blue_mask(&self) -> u32 {
        unsafe {
            (*self.visualtype_ptr).blue_mask
        }
    }

    /// Gets the underlying connection.
    pub fn connection(&self) -> &Rc<Connection> {
        &self.connection
    }

    /// Gets the X11 visual depth.
    pub fn depth(&self) -> u8 {
        self.depth
    }

    /// Gets the green mask.
    pub fn green_mask(&self) -> u32 {
        unsafe {
            (*self.visualtype_ptr).green_mask
        }
    }

    /// Gets red mask.
    pub fn red_mask(&self) -> u32 {
        unsafe {
            (*self.visualtype_ptr).red_mask
        }
    }

    /// Returns the screen number that owns the visual.
    pub fn screen_num(&self) -> u8 {
        self.screen_num
    }

    /// Gets the X11 visual class.
    pub fn visual_class(&self) -> VisualClass {
        unsafe {
            VisualClass::try_from((*self.visualtype_ptr)._class).unwrap()
        }
    }

    /// Gets the X11 visual ID.
    pub fn visual_id(&self) -> u32 {
        unsafe {
            (*self.visualtype_ptr).visual_id
        }
    }

    /// Returns the underlying XCB visual type pointer.
    pub fn xcb_visualtype_ptr(&self) -> *mut xcb_sys::xcb_visualtype_t {
        self.visualtype_ptr
    }
}

impl PixelFormat {
    pub(crate) unsafe fn new(connection: &Rc<Connection>, screen_num: u8, depth: u8,
                             visualtype_ptr: *mut xcb_sys::xcb_visualtype_t) -> PixelFormat
    {
        PixelFormat {
            connection: connection.clone(),
            depth,
            screen_num,
            visualtype_ptr,
        }
    }
}

impl Eq for PixelFormat {}

impl IPixelFormat for PixelFormat {}

impl PartialEq for PixelFormat {
    fn eq(&self, rhs: &PixelFormat) -> bool {
        self.visualtype_ptr == rhs.visualtype_ptr
    }
}

/// Enumeration of X11 visual classes.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum VisualClass {
    StaticGray = 0,
    GrayScale = 1,
    StaticColor = 2,
    PseudoColor = 3,
    TrueColor = 4,
    DirectColor = 5,
}

impl TryFrom<u8> for VisualClass {
    type Error = InvalidVisualClass;

    fn try_from(value: u8) -> Result<VisualClass, InvalidVisualClass> {
        match value {
            0 => Ok(VisualClass::StaticGray),
            1 => Ok(VisualClass::GrayScale),
            2 => Ok(VisualClass::StaticColor),
            3 => Ok(VisualClass::PseudoColor),
            4 => Ok(VisualClass::TrueColor),
            5 => Ok(VisualClass::DirectColor),
            _ => Err(InvalidVisualClass),
        }
    }
}

/// Error indicating an invalid X11 visual class value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidVisualClass;

impl InvalidVisualClass {
    const BRIEF: &'static str = "invalid X11 visual class";
}

impl Display for InvalidVisualClass {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(InvalidVisualClass::BRIEF)
    }
}

impl std::error::Error for InvalidVisualClass {
    fn description(&self) -> &str {
        InvalidVisualClass::BRIEF
    }
}
