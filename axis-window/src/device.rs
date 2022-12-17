/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::any::Any;
use std::rc::Rc;

use crate::context::IContext;
use crate::pixel_format::{PixelFormat, PixelFormats};
use crate::window::WindowBuilder;

/// Trait for window system devices.
pub trait IDevice: 'static + Clone + Eq {
    type Context: IContext<Device = Self>;

    /// Returns the default pixel format for windows on this device.
    fn default_pixel_format(&self) -> <Self::Context as IContext>::PixelFormat;

    /// Constructs a window builder.
    fn new_window(&self) -> <Self::Context as IContext>::WindowBuilder;

    /// Returns an iterator over all available pixel formats.
    fn pixel_formats(&self) -> <Self::Context as IContext>::PixelFormats;
}

/// Wrapper trait which allows an `IDevice` object to be boxed.
pub trait IAnyDevice: Any {
    type WindowId: 'static + Clone;

    fn any(&self) -> &dyn Any;
    fn default_pixel_format(&self) -> PixelFormat;
    fn eq(&self, rhs: &dyn IAnyDevice<WindowId = Self::WindowId>) -> bool;
    fn new_window(&self) -> WindowBuilder<Self::WindowId>;
    fn pixel_formats(&self) -> PixelFormats;
}

impl<T: IDevice> IAnyDevice for T {
    type WindowId = <T::Context as IContext>::WindowId;

    fn any(&self) -> &dyn Any {
        self
    }

    fn default_pixel_format(&self) -> PixelFormat {
        PixelFormat(Rc::new(IDevice::default_pixel_format(self)))
    }

    fn eq(&self, rhs: &dyn IAnyDevice<WindowId = Self::WindowId>) -> bool {
        match rhs.any().downcast_ref::<Self>() {
            None => false,
            Some(rhs) => *self == *rhs,
        }
    }

    fn new_window(&self) -> WindowBuilder<Self::WindowId> {
        WindowBuilder(Box::new(IDevice::new_window(self)))
    }

    fn pixel_formats(&self) -> PixelFormats {
        PixelFormats(Box::new(
            IDevice::pixel_formats(self).map(|pixel_format| PixelFormat(Rc::new(pixel_format))),
        ))
    }
}

/// Window system device type. This is a boxed wrapper around an [`IDevice`] object.
#[derive(Clone)]
pub struct Device<W: 'static + Clone>(pub(crate) Rc<dyn 'static + IAnyDevice<WindowId = W>>);

impl<W: 'static + Clone> Device<W> {
    /// Returns the default pixel format for windows on this device.
    pub fn default_pixel_format(&self) -> PixelFormat {
        self.0.default_pixel_format()
    }

    /// Constructs a window builder.
    pub fn new_window(&self) -> WindowBuilder<W> {
        self.0.new_window()
    }

    /// Returns an iterator over available pixel formats.
    pub fn pixel_formats(&self) -> PixelFormats {
        self.0.pixel_formats()
    }
}

impl<W: 'static + Clone> Eq for Device<W> {}

impl<W: 'static + Clone> PartialEq for Device<W> {
    fn eq(&self, rhs: &Device<W>) -> bool {
        self.0.eq(rhs.0.as_ref())
    }
}

/// Iterator over available window system devices.
pub struct Devices<W: 'static + Clone>(pub(crate) Box<dyn 'static + Iterator<Item = Device<W>>>);

impl<W: 'static + Clone> Iterator for Devices<W> {
    type Item = Device<W>;

    fn next(&mut self) -> Option<Device<W>> {
        self.0.next()
    }
}
