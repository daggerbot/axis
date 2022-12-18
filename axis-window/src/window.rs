/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use math::Vector2;

use crate::error::Result;
use crate::system::ISystem;
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
    /// Allow the window system to choose where to place a window.
    Default,

    /// Centers the window on the default monitor. Programs should not rely on this working exactly
    /// as expected. Typically used for splash windows.
    Centered,

    /// Specifies where the window's top-left corner should appear relative to some undefined
    /// location. This is typically used when a program saves the window's position for the next
    /// time the program is launched.
    Point(Vector2<Coord>),
}

impl Default for WindowPos {
    fn default() -> WindowPos {
        WindowPos::Default
    }
}

/// Interface for window builders. We can't simply have one window builder type that works for all
/// platforms because some platforms may define their own properties that can be specified at
/// creation time.
pub trait IWindowBuilder {
    type System: ISystem<WindowBuilder = Self>;

    /// Builds a window and gives it an ID. The ID does not have to be unique, but it is the value
    /// that the library user will get back when receiving window events.
    fn build(&self, id: <Self::System as ISystem>::WindowId)
        -> Result<<Self::System as ISystem>::Window>;

    /// Sets the initial window position.
    fn with_pos(&mut self, pos: WindowPos) -> &mut Self;

    /// Sets the initial size of the window's client area.
    fn with_size(&mut self, size: Option<Vector2<Coord>>) -> &mut Self;

    /// Sets the initial window title.
    fn with_title<S: Into<String>>(&mut self, title: S) -> &mut Self;

    /// Sets the window's initial visibility state.
    fn with_visibility(&mut self, visible: bool) -> &mut Self;
}

/// Wrapper trait which allows an `IWindowBuilder` object to be boxed.
pub trait IAnyWindowBuilder {
    type WindowId: 'static + Clone;

    fn build(&self, id: Self::WindowId) -> Result<Window<Self::WindowId>>;
    fn set_pos(&mut self, pos: WindowPos);
    fn set_size(&mut self, size: Option<Vector2<Coord>>);
    fn set_title(&mut self, title: String);
    fn set_visible(&mut self, visible: bool);
}

impl<T: IWindowBuilder> IAnyWindowBuilder for T {
    type WindowId = <T::System as ISystem>::WindowId;

    fn build(&self, id: Self::WindowId) -> Result<Window<Self::WindowId>> {
        Ok(Window(Box::new(IWindowBuilder::build(self, id)?)))
    }

    fn set_pos(&mut self, pos: WindowPos) {
        IWindowBuilder::with_pos(self, pos);
    }

    fn set_size(&mut self, size: Option<Vector2<Coord>>) {
        IWindowBuilder::with_size(self, size);
    }

    fn set_title(&mut self, title: String) {
        IWindowBuilder::with_title(self, title);
    }

    fn set_visible(&mut self, visible: bool) {
        IWindowBuilder::with_visibility(self, visible);
    }
}

/// Window builder type.
pub struct WindowBuilder<W: 'static + Clone>(
    pub(crate) Box<dyn 'static + IAnyWindowBuilder<WindowId = W>>,
);

impl<W: 'static + Clone> WindowBuilder<W> {
    /// Builds a window and gives it an ID. The ID does not have to be unique, but it is the value
    /// that the library user will get back when receiving window events.
    pub fn build(&self, id: W) -> Result<Window<W>> {
        self.0.build(id)
    }

    /// Sets the initial window position to the center of the default monitor.
    pub fn centered(&mut self) -> &mut WindowBuilder<W> {
        self.0.set_pos(WindowPos::Centered);
        self
    }

    /// Shows the window as soon as it is created.
    pub fn visible(&mut self) -> &mut WindowBuilder<W> {
        self.0.set_visible(true);
        self
    }

    /// Sets the initial window position.
    pub fn with_pos(&mut self, pos: Vector2<Coord>) -> &mut WindowBuilder<W> {
        self.0.set_pos(WindowPos::Point(pos));
        self
    }

    /// Sets the initial window size.
    pub fn with_size(&mut self, size: Vector2<Coord>) -> &mut WindowBuilder<W> {
        self.0.set_size(Some(size));
        self
    }

    /// Sets the initial window title.
    pub fn with_title<S: Into<String>>(&mut self, title: S) -> &mut WindowBuilder<W> {
        self.0.set_title(title.into());
        self
    }
}

/// Interface for top-level windows.
pub trait IWindow {
    type System: ISystem<Window = Self>;

    /// Destroys the window.
    fn destroy(&self);

    /// Returns the window ID.
    fn id(&self) -> &<Self::System as ISystem>::WindowId;

    /// Returns true if the window is still alive (to the best of our current knowledge).
    fn is_alive(&self) -> bool;

    /// Returns true if the window is visible.
    fn is_visible(&self) -> bool;

    /// Returns the window's current position.
    fn pos(&self) -> Result<Vector2<Coord>>;

    /// Moves the window.
    fn set_pos(&self, pos: Vector2<Coord>) -> Result<()>;

    /// Resizes the window.
    fn set_size(&self, size: Vector2<Coord>) -> Result<()>;

    /// Changes the window title.
    fn set_title(&self, title: &str) -> Result<()>;

    /// Shows or hides the window.
    fn set_visible(&self, visible: bool) -> Result<()>;

    /// Returns the size of the window's client area in pixels.
    fn size(&self) -> Result<Vector2<Coord>>;

    /// Returns the window title.
    fn title(&self) -> Result<String>;
}

/// Wrapper trait which allows an `IWindow` object to be boxed.
pub trait IAnyWindow {
    type WindowId: 'static + Clone;

    fn destroy(&self);
    fn id(&self) -> &Self::WindowId;
    fn is_alive(&self) -> bool;
    fn is_visible(&self) -> bool;
    fn pos(&self) -> Result<Vector2<Coord>>;
    fn set_pos(&self, pos: Vector2<Coord>) -> Result<()>;
    fn set_size(&self, size: Vector2<Coord>) -> Result<()>;
    fn set_title(&self, title: &str) -> Result<()>;
    fn set_visible(&self, visible: bool) -> Result<()>;
    fn size(&self) -> Result<Vector2<Coord>>;
    fn title(&self) -> Result<String>;
}

impl<T: IWindow> IAnyWindow for T {
    type WindowId = <T::System as ISystem>::WindowId;

    fn destroy(&self) {
        IWindow::destroy(self);
    }

    fn id(&self) -> &Self::WindowId {
        IWindow::id(self)
    }

    fn is_alive(&self) -> bool {
        IWindow::is_alive(self)
    }

    fn is_visible(&self) -> bool {
        IWindow::is_visible(self)
    }

    fn pos(&self) -> Result<Vector2<Coord>> {
        IWindow::pos(self)
    }

    fn set_pos(&self, pos: Vector2<Coord>) -> Result<()> {
        IWindow::set_pos(self, pos)
    }

    fn set_size(&self, size: Vector2<Coord>) -> Result<()> {
        IWindow::set_size(self, size)
    }

    fn set_title(&self, title: &str) -> Result<()> {
        IWindow::set_title(self, title)
    }

    fn set_visible(&self, visible: bool) -> Result<()> {
        IWindow::set_visible(self, visible)
    }

    fn size(&self) -> Result<Vector2<Coord>> {
        IWindow::size(self)
    }

    fn title(&self) -> Result<String> {
        IWindow::title(self)
    }
}

/// Top-level window type.
pub struct Window<W: 'static + Clone>(pub(crate) Box<dyn 'static + IAnyWindow<WindowId = W>>);

impl<W: 'static + Clone> Window<W> {
    /// Destroys the window.
    pub fn destroy(&self) {
        self.0.destroy();
    }

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

    /// Returns the window's current position.
    pub fn pos(&self) -> Result<Vector2<Coord>> {
        self.0.pos()
    }

    /// Moves the window.
    pub fn set_pos(&self, pos: Vector2<Coord>) -> Result<()> {
        self.0.set_pos(pos)
    }

    /// Resizes the window.
    pub fn set_size(&self, size: Vector2<Coord>) -> Result<()> {
        self.0.set_size(size)
    }

    /// Changes the window title.
    pub fn set_title(&self, title: &str) -> Result<()> {
        self.0.set_title(title)
    }

    /// Shows or hides the window.
    pub fn set_visible(&self, visible: bool) -> Result<()> {
        self.0.set_visible(visible)
    }

    /// Returns the size of the window's client area in pixels.
    pub fn size(&self) -> Result<Vector2<Coord>> {
        self.0.size()
    }

    /// Returns the window's title.
    pub fn title(&self) -> Result<String> {
        self.0.title()
    }
}
