/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use math::Vector2;

use crate::context::IContext;
use crate::error::Result;
use crate::Coord;

/// Determines the appearance and behavior of a window.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WindowKind {
    Normal,
}

impl Default for WindowKind {
    fn default() -> WindowKind {
        WindowKind::Normal
    }
}

/// Determines where to place a new window.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum WindowPos {
    Default,
    Centered,
    Point(Vector2<Coord>),
}

impl Default for WindowPos {
    fn default() -> WindowPos {
        WindowPos::Default
    }
}

/// Interface for window builders.
pub trait IWindowBuilder {
    type Context: IContext<WindowBuilder = Self>;

    /// Builds a window and gives it an ID. The ID does not have to be unique, but it is the value
    /// that the library user will get back when receiving window events.
    fn build(
        &self, id: <Self::Context as IContext>::WindowId,
    ) -> Result<<Self::Context as IContext>::Window>;
}

/// Object interface for window builders.
pub trait IAnyWindowBuilder {
    type WindowId: 'static + Clone;

    fn build(&self, id: Self::WindowId) -> Result<Window<Self::WindowId>>;
}

impl<T: IWindowBuilder> IAnyWindowBuilder for T {
    type WindowId = <T::Context as IContext>::WindowId;

    fn build(&self, id: Self::WindowId) -> Result<Window<Self::WindowId>> {
        Ok(Window(Box::new(IWindowBuilder::build(self, id)?)))
    }
}

/// Opaque window builder type.
pub struct WindowBuilder<W: 'static + Clone>(
    pub(crate) Box<dyn 'static + IAnyWindowBuilder<WindowId = W>>,
);

impl<W: 'static + Clone> WindowBuilder<W> {
    /// Builds a window and gives it an ID. The ID does not have to be unique, but it is the value
    /// that the library user will get back when receiving window events.
    pub fn build(&self, id: W) -> Result<Window<W>> {
        self.0.build(id)
    }
}

/// Interface for top-level windows.
pub trait IWindow {
    type Context: IContext<Window = Self>;

    /// Returns the window ID.
    fn id(&self) -> &<Self::Context as IContext>::WindowId;

    /// Returns true if the window is still alive (to the best of our current knowledge).
    fn is_alive(&self) -> bool;

    /// Returns true if the window is visible.
    fn is_visible(&self) -> bool;
}

/// Object interface for top-level windows.
pub trait IAnyWindow {
    type WindowId: 'static + Clone;

    fn id(&self) -> &Self::WindowId;
    fn is_alive(&self) -> bool;
    fn is_visible(&self) -> bool;
}

impl<T: IWindow> IAnyWindow for T {
    type WindowId = <T::Context as IContext>::WindowId;

    fn id(&self) -> &Self::WindowId {
        IWindow::id(self)
    }

    fn is_alive(&self) -> bool {
        IWindow::is_alive(self)
    }

    fn is_visible(&self) -> bool {
        IWindow::is_visible(self)
    }
}

/// Opaque top-level window type.
pub struct Window<W: 'static + Clone>(pub(crate) Box<dyn 'static + IAnyWindow<WindowId = W>>);

impl<W: 'static + Clone> Window<W> {
    /// Returns the window ID.
    pub fn id(&self) -> &W {
        self.0.id()
    }

    /// Determines whether the window is alive (to the best of our current knowledge).
    pub fn is_alive(&self) -> bool {
        self.0.is_alive()
    }

    /// Determines whether the window is visible.
    pub fn is_visible(&self) -> bool {
        self.0.is_visible()
    }
}
