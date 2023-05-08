/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::client::{Client, IClient};
use crate::error::Result;

/// Window builder interface.
pub trait IWindowBuilder {
    type Client: IClient;

    /// Builds a window.
    fn build(&self, id: <Self::Client as IClient>::WindowId)
        -> Result<<Self::Client as IClient>::Window>;
}

/// Internal interface for [WindowBuilder].
trait IWindowBuilderObject<W: 'static + Clone>: 'static {
    fn build(&self, id: W) -> Result<Window<W>>;
}

impl<T: 'static + IWindowBuilder> IWindowBuilderObject<<T::Client as IClient>::WindowId> for T {
    fn build(&self, id: <T::Client as IClient>::WindowId)
        -> Result<Window<<T::Client as IClient>::WindowId>>
    {
        Ok(Window::new(<Self as IWindowBuilder>::build(self, id)?))
    }
}

/// Boxed window builder type.
pub struct WindowBuilder<W: 'static + Clone> {
    inner: Box<dyn IWindowBuilderObject<W>>,
}

impl<W: 'static + Clone> WindowBuilder<W> {
    pub(crate) fn new<T: 'static + IWindowBuilder>(inner: T) -> WindowBuilder<W>
    where <T as IWindowBuilder>::Client: IClient<WindowId = W>
    {
        WindowBuilder { inner: Box::new(inner) }
    }
}

impl<W: 'static + Clone> IWindowBuilder for WindowBuilder<W> {
    type Client = Client<W>;

    fn build(&self, id: W) -> Result<Window<W>> {
        self.inner.build(id)
    }
}

/// Window interface.
pub trait IWindow {
    type Client: IClient;

    /// Destroys the window.
    fn destroy(&self);

    /// Returns the window ID which is used when reporting events.
    fn id(&self) -> &<Self::Client as IClient>::WindowId;

    /// Returns true if the window is visible.
    fn is_visible(&self) -> bool;

    /// Shows or hides the window.
    fn set_visible(&self, visible: bool) -> Result<()>;
}

/// Internal interface for [Window].
trait IWindowObject<W: 'static + Clone>: 'static {
    fn destroy(&self);
    fn id(&self) -> &W;
    fn is_visible(&self) -> bool;
    fn set_visible(&self, visible: bool) -> Result<()>;
}

impl<T: 'static + IWindow> IWindowObject<<T::Client as IClient>::WindowId> for T {
    fn destroy(&self) {
        <T as IWindow>::destroy(self)
    }

    fn id(&self) -> &<T::Client as IClient>::WindowId {
        <T as IWindow>::id(self)
    }

    fn is_visible(&self) -> bool {
        <T as IWindow>::is_visible(self)
    }

    fn set_visible(&self, visible: bool) -> Result<()> {
        <T as IWindow>::set_visible(self, visible)
    }
}

/// Boxed window type.
pub struct Window<W: 'static + Clone> {
    inner: Box<dyn IWindowObject<W>>,
}

impl<W: 'static + Clone> Window<W> {
    fn new<T: 'static + IWindow>(inner: T) -> Window<W>
    where T::Client: IClient<WindowId = W>
    {
        Window { inner: Box::new(inner) }
    }
}

impl<W: 'static + Clone> IWindow for Window<W> {
    type Client = Client<W>;

    fn destroy(&self) {
        self.inner.destroy()
    }

    fn id(&self) -> &W {
        self.inner.id()
    }

    fn is_visible(&self) -> bool {
        self.inner.is_visible()
    }

    fn set_visible(&self, visible: bool) -> Result<()> {
        self.inner.set_visible(visible)
    }
}
