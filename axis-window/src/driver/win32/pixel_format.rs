/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::ffi::c_int;

use crate::pixel_format::IPixelFormat;

/// Win32 pixel format type.
#[derive(Clone)]
pub enum PixelFormat {
    Default,
    Gdi {
        index: c_int,
        pfd: winapi::um::wingdi::PIXELFORMATDESCRIPTOR,
    },
}

impl Default for PixelFormat {
    fn default() -> PixelFormat {
        PixelFormat::Default
    }
}

impl Eq for PixelFormat {}

impl IPixelFormat for PixelFormat {}

impl PartialEq for PixelFormat {
    fn eq(&self, rhs: &PixelFormat) -> bool {
        match (self, rhs) {
            (&PixelFormat::Default, &PixelFormat::Default) => true,
            (
                &PixelFormat::Gdi {
                    index: index0,
                    pfd: pfd0,
                },
                &PixelFormat::Gdi {
                    index: index1,
                    pfd: pfd1,
                },
            ) => {
                index0 == index1
                    && pfd0.nSize == pfd1.nSize
                    && pfd0.nVersion == pfd1.nVersion
                    && pfd0.dwFlags == pfd1.dwFlags
                    && pfd0.iPixelType == pfd1.iPixelType
                    && pfd0.cColorBits == pfd1.cColorBits
                    && pfd0.cRedBits == pfd1.cRedBits
                    && pfd0.cRedShift == pfd1.cRedShift
                    && pfd0.cGreenBits == pfd1.cGreenBits
                    && pfd0.cGreenShift == pfd1.cGreenShift
                    && pfd0.cBlueBits == pfd1.cBlueBits
                    && pfd0.cBlueShift == pfd1.cBlueShift
                    && pfd0.cAlphaBits == pfd1.cAlphaBits
                    && pfd0.cAlphaShift == pfd1.cAlphaShift
                    && pfd0.cAccumBits == pfd1.cAccumBits
                    && pfd0.cAccumRedBits == pfd1.cAccumRedBits
                    && pfd0.cAccumGreenBits == pfd1.cAccumGreenBits
                    && pfd0.cAccumBlueBits == pfd1.cAccumBlueBits
                    && pfd0.cAccumAlphaBits == pfd1.cAccumAlphaBits
                    && pfd0.cDepthBits == pfd1.cDepthBits
                    && pfd0.cStencilBits == pfd1.cStencilBits
                    && pfd0.cAuxBuffers == pfd1.cAuxBuffers
                    && pfd0.iLayerType == pfd1.iLayerType
                    && pfd0.bReserved == pfd1.bReserved
                    && pfd0.dwLayerMask == pfd1.dwLayerMask
                    && pfd0.dwVisibleMask == pfd1.dwVisibleMask
                    && pfd0.dwDamageMask == pfd1.dwDamageMask
            },
            _ => false,
        }
    }
}
