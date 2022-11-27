/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::rc::Rc;

use crate::driver::win32::util::Win32Error;
use crate::driver::win32::window::WindowShared;
use crate::error::Result;

/// Wrapper around an `HDC`.
pub struct Dc<W: 'static + Clone> {
    hdc: winapi::shared::windef::HDC,
    window: Rc<WindowShared<W>>,
}

impl<W: 'static + Clone> Dc<W> {
    pub fn get(window: &Rc<WindowShared<W>>) -> Result<Dc<W>> {
        let hwnd = window.try_hwnd()?;
        let hdc;

        unsafe {
            hdc = winapi::um::winuser::GetDC(hwnd);
        }

        if hdc.is_null() {
            return Err(err!(SystemError("GetDC"): Win32Error::last()));
        }

        Ok(Dc {
            hdc,
            window: window.clone(),
        })
    }

    pub fn hdc(&self) -> winapi::shared::windef::HDC { self.hdc }
}

impl<W: 'static + Clone> Drop for Dc<W> {
    fn drop(&mut self) {
        if let Ok(hwnd) = self.window.try_hwnd() {
            unsafe {
                winapi::um::winuser::ReleaseDC(hwnd, self.hdc);
            }
        }
    }
}
