/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::cell::Cell;
use std::rc::Rc;

use crate::device::{Device, Devices, IDevice};
use crate::error::Result;
use crate::event::{Event, UpdateKind};
use crate::pixel_format::IPixelFormat;
use crate::window::{IWindow, IWindowBuilder};

/// Window system context trait.
pub trait ISystem: 'static + Sized {
    type Device: IDevice<System = Self>;
    type Devices: Iterator<Item = Self::Device>;
    type PixelFormat: IPixelFormat;
    type PixelFormats: Iterator<Item = Self::PixelFormat>;
    type Window: IWindow<System = Self>;
    type WindowBuilder: IWindowBuilder<System = Self>;
    type WindowId: 'static + Clone;

    /// Returns the default device.
    fn default_device(&self) -> Self::Device;

    /// Returns an iterator over all available devices.
    fn devices(&self) -> Self::Devices;

    /// Runs the main loop.
    fn run<F: Fn(Event<Self::WindowId>)>(&self, main_loop: &MainLoop, f: F) -> Result<()>;
}

/// Wrapper trait which allows an [`ISystem`] object to be boxed.
pub trait IAnySystem {
    type WindowId: 'static + Clone;

    fn default_device(&self) -> Device<Self::WindowId>;
    fn devices(&self) -> Devices<Self::WindowId>;
    fn run(&self, main_loop: &MainLoop, f: &dyn Fn(Event<Self::WindowId>)) -> Result<()>;
}

impl<T: ISystem> IAnySystem for T {
    type WindowId = <T as ISystem>::WindowId;

    fn default_device(&self) -> Device<Self::WindowId> {
        Device(Rc::new(ISystem::default_device(self)))
    }

    fn devices(&self) -> Devices<Self::WindowId> {
        Devices(Box::new(ISystem::devices(self).map(|device| Device(Rc::new(device)))))
    }

    fn run(&self, main_loop: &MainLoop, f: &dyn Fn(Event<Self::WindowId>)) -> Result<()> {
        ISystem::run(self, main_loop, f)
    }
}

/// Window system context type. This is a boxed wrapper around an [`IContext`] object.
pub struct System<W: 'static + Clone>(pub(crate) Box<dyn 'static + IAnySystem<WindowId = W>>);

impl<W: 'static + Clone> System<W> {
    /// Returns the default device.
    pub fn default_device(&self) -> Device<W> {
        self.0.default_device()
    }

    /// Returns an iterator over all available devices.
    pub fn devices(&self) -> Devices<W> {
        self.0.devices()
    }

    /// Opens a context for the default window system.
    #[allow(unreachable_code)]
    pub fn open_default() -> Result<System<W>> {
        #[cfg(all(feature = "win32-driver", target_os = "windows"))]
        {
            return Ok(System(Box::new(crate::driver::win32::System::open()?)));
        }

        #[cfg(x11_enabled)]
        {
            return Ok(System(Box::new(
                crate::driver::x11::System::open_default()?,
            )));
        }

        Err(err!(UnsupportedPlatform))
    }

    /// Runs the main loop.
    pub fn run<F: Fn(Event<W>)>(&self, main_loop: &MainLoop, f: F) -> Result<()> {
        self.0.run(main_loop, &f)
    }
}

impl<W: 'static + Clone, C: ISystem<WindowId = W>> From<C> for System<W> {
    /// Constructs an opaque window system context.
    fn from(inner: C) -> System<W> {
        System(Box::new(inner))
    }
}

/// Object which determines the behavior of the main loop and when it breaks.
pub struct MainLoop {
    quit: Cell<bool>,
    update_kind: Cell<UpdateKind>,
}

impl MainLoop {
    /// Constructs a new main loop.
    pub fn new(update_kind: UpdateKind) -> MainLoop {
        MainLoop {
            quit: Cell::new(false),
            update_kind: Cell::new(update_kind),
        }
    }

    /// Causes the main loop to break as soon as possible.
    pub fn quit(&self) {
        self.quit.set(true);
    }

    /// Changes the behavior of update events.
    pub fn set_update_kind(&self, update_kind: UpdateKind) {
        self.update_kind.set(update_kind);
    }

    /// Returns the update event behavior.
    pub fn update_kind(&self) -> UpdateKind {
        self.update_kind.get()
    }
}

impl MainLoop {
    pub(crate) fn clear_quit_flag(&self) {
        self.quit.set(false);
    }

    pub(crate) fn is_quit_requested(&self) -> bool {
        self.quit.get()
    }
}
