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

use crate::device::IDevice;
use crate::driver::win32::event::EventManager;
use crate::driver::win32::pixel_format::PixelFormat;
use crate::driver::win32::system::System;
use crate::driver::win32::window::WindowBuilder;

/// Iterator over available Win32 window system devices (there is only one).
pub type Devices<W> = Once<Device<W>>;

/// Win32 window system device type.
#[derive(Clone)]
pub struct Device<W: 'static + Clone> {
    event_manager: Rc<EventManager<W>>,
    _phantom: PhantomData<W>,
    unique: Rc<()>,
}

impl<W: 'static + Clone> Device<W> {
    pub(crate) fn event_manager(&self) -> &Rc<EventManager<W>> {
        &self.event_manager
    }

    pub(crate) fn new(system: &System<W>) -> Device<W> {
        Device {
            event_manager: system.event_manager().clone(),
            _phantom: PhantomData,
            unique: system.unique().clone(),
        }
    }
}

impl<W: 'static + Clone> Eq for Device<W> {}

impl<W: 'static + Clone> IDevice for Device<W> {
    type System = System<W>;

    fn default_pixel_format(&self) -> PixelFormat {
        PixelFormat::Default
    }

    fn new_window(&self) -> WindowBuilder<W> {
        WindowBuilder::new(self)
    }

    fn pixel_formats(&self) -> Once<PixelFormat> {
        std::iter::once(PixelFormat::Default)
    }
}

impl<W: 'static + Clone> PartialEq for Device<W> {
    fn eq(&self, rhs: &Device<W>) -> bool {
        &*self.unique as *const () == &*rhs.unique as *const ()
    }
}
