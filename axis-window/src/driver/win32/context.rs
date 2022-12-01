/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::iter::Once;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::context::IContext;
use crate::driver::win32::device::{Device, Devices};
use crate::driver::win32::pixel_format::PixelFormat;
use crate::driver::win32::window::{Window, WindowBuilder};
use crate::error::Result;

/// Context to the Win32 window system.
pub struct Context<W: 'static + Clone> {
    _phantom: PhantomData<W>,
    unique: Rc<()>,
}

impl<W: 'static + Clone> Context<W> {
    /// Opens a window system context for the current thread.
    ///
    /// It should be noted that multiple contexts may be opened on the same thread, but polling
    /// events from one may discard events for windows created by another.
    pub fn open() -> Result<Context<W>> {
        Ok(Context {
            _phantom: PhantomData,
            unique: Rc::new(()),
        })
    }
}

impl<W: 'static + Clone> Context<W> {
    /// Returns an [`Rc`] that is unique to this context.
    pub(crate) fn unique(&self) -> &Rc<()> {
        &self.unique
    }
}

impl<W: 'static + Clone> IContext for Context<W> {
    type Device = Device<W>;
    type Devices = Devices<W>;
    type PixelFormat = PixelFormat;
    type PixelFormats = Once<PixelFormat>;
    type Window = Window<W>;
    type WindowBuilder = WindowBuilder<W>;
    type WindowId = W;

    fn default_device(&self) -> Device<W> {
        Device::new(self)
    }

    fn devices(&self) -> Devices<W> {
        std::iter::once(Device::new(self))
    }
}

impl<W: 'static + Clone> Eq for Context<W> {}

impl<W: 'static + Clone> PartialEq for Context<W> {
    fn eq(&self, rhs: &Context<W>) -> bool {
        &*self.unique as *const () == &*rhs.unique as *const ()
    }
}
