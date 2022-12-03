/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::rc::Rc;

use crate::driver::x11::connection::Connection;
use crate::driver::x11::device::Device;
use crate::pixel_format::IPixelFormat;

/// X11 pixel format (visual) type.
#[derive(Clone)]
pub struct PixelFormat {
    connection: Rc<Connection>,
    depth: u8,
    screen_index: u8,
    visual_ptr: *mut xcb_sys::xcb_visualtype_t,
}

impl PixelFormat {
    /// Returns the underlying X connection.
    pub fn connection(&self) -> &Rc<Connection> {
        &self.connection
    }

    /// Returns the depth of the visual.
    pub fn depth(&self) -> u8 {
        self.depth
    }

    /// Returns the index of the screen that visual pertains to.
    pub fn screen_index(&self) -> u8 {
        self.screen_index
    }

    /// Returns the visual's XID.
    pub fn visual_id(&self) -> u32 {
        unsafe { (*self.visual_ptr).visual_id }
    }
}

impl Eq for PixelFormat {}

impl IPixelFormat for PixelFormat {}

impl PartialEq for PixelFormat {
    fn eq(&self, rhs: &PixelFormat) -> bool {
        self.visual_ptr == rhs.visual_ptr
            // Not sure if this is necessary. Can different X screens have the same visual
            // available? Would XCB give different structs if they did? Let's leave this until we've
            // determined that it's definitely unnecessary.
            && self.screen_index == rhs.screen_index
    }
}

/// Iterator over available X11 pixel formats.
pub struct PixelFormats {
    connection: Rc<Connection>,
    depth_iter: xcb_sys::xcb_depth_iterator_t,
    screen_index: u8,
    visual_iter: xcb_sys::xcb_visualtype_iterator_t,
}

impl PixelFormats {
    pub(crate) fn new<W: 'static + Clone>(device: &Device<W>) -> PixelFormats {
        let depth_iter;
        let visual_iter;

        unsafe {
            depth_iter = xcb_sys::xcb_screen_allowed_depths_iterator(device.xcb_screen_ptr());
            visual_iter = match depth_iter.rem {
                0 => std::mem::zeroed(),
                _ => xcb_sys::xcb_depth_visuals_iterator(depth_iter.data),
            };
        }

        PixelFormats {
            connection: device.connection().clone(),
            depth_iter,
            screen_index: device.screen_index(),
            visual_iter,
        }
    }
}

impl Iterator for PixelFormats {
    type Item = PixelFormat;

    fn next(&mut self) -> Option<PixelFormat> {
        unsafe {
            if self.visual_iter.rem == 0 {
                if self.depth_iter.rem == 0 {
                    return None;
                }
                xcb_sys::xcb_depth_next(&mut self.depth_iter);
                self.visual_iter = xcb_sys::xcb_depth_visuals_iterator(self.depth_iter.data);
            }

            let pixel_format = PixelFormat {
                connection: self.connection.clone(),
                depth: (*self.depth_iter.data).depth,
                screen_index: self.screen_index,
                visual_ptr: self.visual_iter.data,
            };

            xcb_sys::xcb_visualtype_next(&mut self.visual_iter);
            Some(pixel_format)
        }
    }
}
