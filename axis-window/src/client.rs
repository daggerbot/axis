/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::error::{Error, Result};
use crate::event::{Event, MainLoop};
use crate::pixel_format::{IPixelFormat, PixelFormat};
use crate::window::{IWindow, IWindowBuilder, Window, WindowBuilder};

/// Interface for window system clients.
pub trait IClient {
    type PixelFormat: IPixelFormat;
    type Window: IWindow<Client = Self>;
    type WindowBuilder: IWindowBuilder<Client = Self>;
    type WindowId: 'static + Clone;

    /// Returns the default pixel format.
    fn default_pixel_format(&self) -> Self::PixelFormat;

    /// Runs the main loop.
    fn run<F: Fn(Event<Self::WindowId>)>(&self, main_loop: &MainLoop, f: &F) -> Result<()>;

    /// Returns a new window builder.
    fn window(&self) -> Self::WindowBuilder;
}

/// Internal interface for [Client].
pub trait IClientObject<W: 'static + Clone>: 'static {
    fn default_pixel_format(&self) -> PixelFormat;
    fn run(&self, main_loop: &MainLoop, f: &dyn Fn(Event<W>)) -> Result<()>;
    fn window(&self) -> WindowBuilder<W>;
}

impl<T: 'static + IClient> IClientObject<T::WindowId> for T {
    fn default_pixel_format(&self) -> PixelFormat {
        PixelFormat::new(<T as IClient>::default_pixel_format(&self))
    }

    fn run(&self, main_loop: &MainLoop, f: &dyn Fn(Event<T::WindowId>)) -> Result<()> {
        <T as IClient>::run(self, main_loop, &f)
    }

    fn window(&self) -> WindowBuilder<T::WindowId> {
        WindowBuilder::new(<T as IClient>::window(self))
    }
}

/// Boxed window system client type.
pub struct Client<W: 'static + Clone> {
    inner: Box<dyn IClientObject<W>>,
}

impl<W: 'static + Clone> Client<W> {
    /// Boxes a client object.
    pub fn new<T: 'static + IClient<WindowId = W>>(inner: T) -> Client<W> {
        Client { inner: Box::new(inner) }
    }

    /// Opens a client for the default window system.
    pub fn open_default() -> Result<Client<W>> {
        #[allow(unused_assignments)]
        #[allow(unused_mut)]
        #[allow(unused_variables)]
        let mut err: Option<Error> = None;

        #[cfg(all(feature = "win32-driver", target_os = "windows"))]
        {
            return Ok(Client::new(crate::driver::win32::Client::open()?));
        }

        #[cfg(all(feature = "x11-driver", any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "linux",
            target_os = "netbsd",
            target_os = "openbsd",
        )))]
        {
            err = match crate::driver::x11::Client::open_default() {
                Ok(client) => return Ok(Client::new(client)),
                Err(err) => Some(err),
            };
        }

        #[allow(unreachable_code)]
        Err(match err {
            None => err!(LibraryError("no suitable window system drivers configured")),
            Some(err) => err,
        })
    }
}

impl<W: 'static + Clone> IClient for Client<W> {
    type PixelFormat = PixelFormat;
    type Window = Window<W>;
    type WindowBuilder = WindowBuilder<W>;
    type WindowId = W;

    fn default_pixel_format(&self) -> PixelFormat {
        self.inner.default_pixel_format()
    }

    fn run<F: Fn(Event<W>)>(&self, main_loop: &MainLoop, f: &F) -> Result<()> {
        self.inner.run(main_loop, f)
    }

    fn window(&self) -> WindowBuilder<W> {
        self.inner.window()
    }
}
