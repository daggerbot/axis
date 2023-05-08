/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use libc::size_t;
use winapi::um::wingdi::PIXELFORMATDESCRIPTOR;

use crate::pixel_format::IPixelFormat;

/// Internal data for [PixelFormat].
#[derive(Clone)]
enum PixelFormatData {
    Default,
    #[allow(dead_code)]
    Gdi(i32, PIXELFORMATDESCRIPTOR),
}

impl Default for PixelFormatData {
    fn default() -> PixelFormatData {
        PixelFormatData::Default
    }
}

impl Eq for PixelFormatData {}

impl PartialEq for PixelFormatData {
    fn eq(&self, rhs: &PixelFormatData) -> bool {
        match (self, rhs) {
            (&PixelFormatData::Default, &PixelFormatData::Default) => true,
            (&PixelFormatData::Gdi(index0, ref pfd0), &PixelFormatData::Gdi(index1, ref pfd1)) => {
                let pfd0 = pfd0 as *const PIXELFORMATDESCRIPTOR;
                let pfd1 = pfd1 as *const PIXELFORMATDESCRIPTOR;
                let pfd_size = std::mem::size_of::<PIXELFORMATDESCRIPTOR>();
                if index0 != index1 {
                    return false;
                }

                unsafe {
                    libc::memcmp(pfd0 as *const _, pfd1 as *const _, pfd_size as size_t) == 0
                }
            },
            _ => false,
        }
    }
}

/// Win32 pixel format type.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct PixelFormat {
    data: PixelFormatData,
}

impl PixelFormat {
    /// Returns the GDI pixel format descriptor if `self` is a GDI pixel format.
    pub fn gdi_descriptor(&self) -> Option<&PIXELFORMATDESCRIPTOR> {
        match self.data {
            PixelFormatData::Gdi(_, ref pfd) => Some(pfd),
            _ => None,
        }
    }

    /// Returns the GDI pixel format index if `self` is a GDI pixel format.
    pub fn gdi_index(&self) -> Option<i32> {
        match self.data {
            PixelFormatData::Gdi(index, _) => Some(index),
            _ => None,
        }
    }
}

impl IPixelFormat for PixelFormat {}
