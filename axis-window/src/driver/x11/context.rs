/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::rc::Rc;

use crate::context::{IContext, MainLoop};
use crate::driver::x11::connection::Connection;
use crate::driver::x11::device::{Device, Devices};
use crate::driver::x11::pixel_format::{PixelFormat, PixelFormats};
use crate::driver::x11::window::{Window, WindowBuilder, WindowManager};
use crate::error::Result;
use crate::event::{Event, UpdateKind};
use crate::ffi::CBox;

/// Macro which defines our `Atoms` struct. As the list of atoms we use grows, doing this by hand
/// would become repetative and error-prone.
macro_rules! atoms {
    { $($atom:ident,)* } => {
        /// Commonly used X11 atoms.
        #[allow(non_snake_case)]
        #[derive(Clone, Copy)]
        pub struct Atoms {
            $(pub $atom: u32,)*
        }

        impl Atoms {
            #[allow(non_snake_case)]
            fn init(connection: &Connection) -> Result<Atoms> {
                $(let $atom = connection.intern_atom(stringify!($atom));)*

                Ok(Atoms {
                    $($atom: connection.intern_atom_reply($atom)?,)*
                })
            }
        }
    };
}

atoms! {
    _NET_WM_ICON_NAME,
    _NET_WM_NAME,
    UTF8_STRING,
    WM_DELETE_WINDOW,
    WM_PROTOCOLS,
}

/// X11 window system context.
pub struct Context<W: 'static + Clone> {
    atoms: Rc<Atoms>,
    connection: Rc<Connection>,
    _phantom: PhantomData<W>,
    window_manager: Rc<RefCell<WindowManager<W>>>,
    xcb: *mut xcb_sys::xcb_connection_t,
}

impl<W: 'static + Clone> Context<W> {
    /// Returns the underlying X connection.
    pub fn connection(&self) -> &Rc<Connection> {
        &self.connection
    }

    /// Opens a connection to the specified X server.
    pub fn open<S: Into<Vec<u8>>>(name: S) -> Result<Context<W>> {
        Context::init(Connection::open(name)?)
    }

    /// Opens a connectoin to the default X server.
    pub fn open_default() -> Result<Context<W>> {
        Context::init(Connection::open_default()?)
    }
}

impl<W: 'static + Clone> Context<W> {
    pub(crate) fn atoms(&self) -> &Rc<Atoms> {
        &self.atoms
    }

    pub(crate) fn window_manager(&self) -> &Rc<RefCell<WindowManager<W>>> {
        &self.window_manager
    }
}

impl<W: 'static + Clone> Context<W> {
    /// Safe wrapper around `xcb_flush()` with error checking.
    fn flush(&self) -> Result<()> {
        unsafe {
            xcb_sys::xcb_flush(self.xcb);
            match xcb_sys::xcb_connection_has_error(self.xcb) {
                0 => Ok(()),
                err_code => Err(err!(IoError{"xcb_connection_has_error: {}", err_code})),
            }
        }
    }
}

impl<W: 'static + Clone> Context<W> {
    /// Performs any initialization on the context that occurs after the connection is obtained.
    fn init(connection: Connection) -> Result<Context<W>> {
        #[cfg(feature = "x11-sys")]
        unsafe {
            // We'll use XCB to poll events.
            x11_sys::XSetEventQueueOwner(connection.xlib_display_ptr(),
                                         x11_sys::XEventQueueOwner_XCBOwnsEventQueue);
        }

        let atoms = Atoms::init(&connection)?;
        let xcb = connection.xcb_connection_ptr();

        Ok(Context {
            atoms: Rc::new(atoms),
            connection: Rc::new(connection),
            _phantom: PhantomData,
            window_manager: Rc::new(RefCell::new(WindowManager::new())),
            xcb,
        })
    }
}

impl<W: 'static + Clone> Eq for Context<W> {}

impl<W: 'static + Clone> IContext for Context<W> {
    type Device = Device<W>;
    type Devices = Devices<W>;
    type PixelFormat = PixelFormat;
    type PixelFormats = PixelFormats;
    type Window = Window<W>;
    type WindowBuilder = WindowBuilder<W>;
    type WindowId = W;

    fn default_device(&self) -> Device<W> {
        for device in self.devices() {
            if device.screen_index() == self.connection.default_screen_index() {
                return device;
            }
        }
        panic!("invalid default X screen");
    }

    fn devices(&self) -> Devices<W> {
        Devices::new(&self)
    }

    fn run<F: Fn(Event<Self::WindowId>)>(&self, main_loop: &MainLoop, callback: F) -> Result<()> {
        main_loop.clear_quit_flag();

        // Flag which indicates whether we can emit another Update event. Using this flag prevents
        // two Update events from being emitted in a row when the main loop is passive.
        let update_ready = Cell::new(true);

        // Callback wrapper which sets `update_ready` when called. We use this because we don't
        // otherwise know if the callback was actually invoked when we pass it to `handle_event()`.
        let f = |event| {
            callback(event);
            update_ready.set(true);
        };

        'main_loop: while !main_loop.is_quit_requested() {
            self.flush()?;

            unsafe {
                let mut xevent_ptr = xcb_sys::xcb_poll_for_event(self.xcb);
                if xevent_ptr.is_null() {
                    match main_loop.update_kind() {
                        UpdateKind::Passive => {
                            if update_ready.take() {
                                callback(Event::Update { kind: UpdateKind::Passive });
                                if main_loop.is_quit_requested() {
                                    break 'main_loop;
                                }
                            }
                            self.flush()?;
                            xevent_ptr = xcb_sys::xcb_wait_for_event(self.xcb);
                        },

                        UpdateKind::Active | UpdateKind::VBlank => {
                            // No way (that I'm aware of) to handle VBlank here.
                            callback(Event::Update { kind: UpdateKind::Active });
                            continue 'main_loop;
                        },
                    }
                }

                if !xevent_ptr.is_null() {
                    let xevent = CBox::from_raw(xevent_ptr);
                    self.handle_event(xevent, &f)?;
                }
            }
        }

        Ok(())
    }
}

impl<W: 'static + Clone> PartialEq for Context<W> {
    fn eq(&self, rhs: &Context<W>) -> bool {
        self.connection == rhs.connection
    }
}
