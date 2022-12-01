/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::cell::RefCell;
use std::ffi::{c_char, CString};
use std::marker::PhantomData;
use std::os::unix::io::{AsFd, BorrowedFd};
use std::rc::Rc;

use crate::context::{IContext, MainLoop};
use crate::driver::x11::device::{Device, Devices};
use crate::driver::x11::pixel_format::{PixelFormat, PixelFormats};
use crate::driver::x11::window::{Window, WindowBuilder, WindowManager};
use crate::error::Result;
use crate::event::{Event, UpdateKind};
use crate::util::CBox;

/// Connection to an X11 server.
pub struct Connection {
    default_screen_index: u8,
    xcb: *mut xcb_sys::xcb_connection_t,
    #[cfg(feature = "x11-sys")]
    xlib: *mut x11_sys::Display,
}

impl Connection {
    /// Returns the index of the default screen.
    pub fn default_screen_index(&self) -> u8 {
        self.default_screen_index
    }

    /// Opens a connection to the specified X server.
    ///
    /// If the `x11-sys` feature is enabled, the Xlib API owns the event queue by default.
    pub fn open<S: Into<Vec<u8>>>(name: S) -> Result<Connection> {
        let c_name = CString::new(name)?;

        unsafe { Connection::open_raw(c_name.as_ptr()) }
    }

    /// Opens a connection to the default X server.
    ///
    /// If the `x11-sys` feature is enabled, the Xlib API owns the event queue by default.
    pub fn open_default() -> Result<Connection> {
        unsafe { Connection::open_raw(std::ptr::null()) }
    }

    /// Returns a pointer to the underlying XCB connection.
    pub fn xcb_connection_ptr(&self) -> *mut xcb_sys::xcb_connection_t {
        self.xcb
    }

    /// Returns a pointer to the underlying Xlib display connection.
    #[cfg(feature = "x11-sys")]
    pub fn xlib_display_ptr(&self) -> *mut x11_sys::Display {
        self.xlib
    }
}

impl Connection {
    pub(crate) fn intern_atom<S: ?Sized + AsRef<[u8]>>(
        &self, name: &S,
    ) -> xcb_sys::xcb_intern_atom_cookie_t {
        let name = name.as_ref();

        unsafe {
            xcb_sys::xcb_intern_atom(
                self.xcb,
                0,
                u16::try_from(name.len()).unwrap(),
                name.as_ptr() as *const c_char,
            )
        }
    }

    pub(crate) fn intern_atom_reply(
        &self, cookie: xcb_sys::xcb_intern_atom_cookie_t,
    ) -> Result<u32> {
        unsafe {
            let mut err_ptr = std::ptr::null_mut();
            let reply_ptr = xcb_sys::xcb_intern_atom_reply(self.xcb, cookie, &mut err_ptr);
            if reply_ptr.is_null() {
                if err_ptr.is_null() {
                    return Err(err!(RequestFailed("XInternAtom")));
                } else {
                    let err = CBox::from_raw(err_ptr);
                    return Err(err!(RequestFailed{"XInternAtom: {:?}", *err}));
                }
            }
            let reply = CBox::from_raw(reply_ptr);
            Ok(reply.atom)
        }
    }
}

impl Connection {
    /// Opens a connection to the specified X server.
    #[cfg(not(feature = "x11-sys"))]
    unsafe fn open_raw(name: *const c_char) -> Result<Connection> {
        let mut default_screen_index = 0;
        let xcb = xcb_sys::xcb_connect(name, &mut default_screen_index);
        if xcb.is_null() {
            return Err(err!(ConnectionFailed("xcb_connect failed")));
        }

        Ok(Connection {
            default_screen_index: u8::try_from(default_screen_index)?,
            xcb,
        })
    }

    /// Opens a connection to the specified X server.
    #[cfg(feature = "x11-sys")]
    unsafe fn open_raw(name: *const c_char) -> Result<Connection> {
        let xlib = x11_sys::XOpenDisplay(name);
        if xlib.is_null() {
            return Err(err!(ConnectionFailed("XOpenDisplay failed")));
        }

        Ok(Connection {
            default_screen_index: u8::try_from(x11_sys::XDefaultScreen(xlib))?,
            xcb: x11_sys::XGetXCBConnection(xlib) as *mut xcb_sys::xcb_connection_t,
            xlib,
        })
    }
}

impl AsFd for Connection {
    fn as_fd(&self) -> BorrowedFd {
        unsafe { BorrowedFd::borrow_raw(xcb_sys::xcb_get_file_descriptor(self.xcb)) }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        #[cfg(not(feature = "x11-sys"))]
        unsafe {
            xcb_sys::xcb_disconnect(self.xcb);
        }

        #[cfg(feature = "x11-sys")]
        unsafe {
            x11_sys::XCloseDisplay(self.xlib);
        }
    }
}

impl Eq for Connection {}

impl PartialEq for Connection {
    fn eq(&self, rhs: &Connection) -> bool {
        self as *const Connection == rhs as *const Connection
    }
}

/// Commonly used X atoms.
#[derive(Clone, Copy)]
pub struct Atoms {
    pub wm_delete_window: u32,
    pub wm_protocols: u32,
}

impl Atoms {
    fn intern(connection: &Connection) -> Result<Atoms> {
        let wm_delete_window = connection.intern_atom("WM_DELETE_WINDOW");
        let wm_protocols = connection.intern_atom("WM_PROTOCOLS");

        Ok(Atoms {
            wm_delete_window: connection.intern_atom_reply(wm_delete_window)?,
            wm_protocols: connection.intern_atom_reply(wm_protocols)?,
        })
    }
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
    fn flush(&self) -> Result<()> {
        unsafe {
            xcb_sys::xcb_flush(self.xcb);
            match xcb_sys::xcb_connection_has_error(self.xcb) {
                0 => Ok(()),
                err_code => Err(err!(IoError{"xcb_connection_has_error: {}", err_code})),
            }
        }
    }

    fn handle_client_message<F: FnMut(Event<W>)>(
        &self, event: &xcb_sys::xcb_client_message_event_t, f: &mut F,
    ) -> Result<()> {
        if event.type_ == self.atoms.wm_protocols && event.format == 32 {
            let protocol = unsafe { event.data.data32[0] };
            if protocol == self.atoms.wm_delete_window {
                if let Some(window) = self.window_manager.borrow().get(event.window).cloned() {
                    f(Event::Close {
                        window_id: window.id().clone(),
                    });
                }
            }
        }

        Ok(())
    }

    fn handle_event<F: FnMut(Event<W>)>(
        &self, xevent: CBox<xcb_sys::xcb_generic_event_t>, f: &mut F,
    ) -> Result<()> {
        let xevent_ref: &xcb_sys::xcb_generic_event_t = xevent.as_ref();
        let xevent_ptr = xevent_ref as *const xcb_sys::xcb_generic_event_t;

        match (xevent.response_type & !0x80) as u32 {
            xcb_sys::XCB_CLIENT_MESSAGE => {
                self.handle_client_message(
                    unsafe { &(*(xevent_ptr as *const xcb_sys::xcb_client_message_event_t)) },
                    f,
                )?;
            },

            xcb_sys::XCB_DESTROY_NOTIFY => {
                let ev = unsafe { *(xevent_ptr as *const xcb_sys::xcb_destroy_notify_event_t) };
                if let Some(window) = self.window_manager.borrow_mut().expire(ev.window) {
                    f(Event::Destroy {
                        window_id: window.id().clone(),
                    });
                }
            },

            xcb_sys::XCB_MAP_NOTIFY => {
                let ev = unsafe { *(xevent_ptr as *const xcb_sys::xcb_map_notify_event_t) };
                if let Some(window) = self.window_manager.borrow().get(ev.window).cloned() {
                    window.update_visibility(true);
                    f(Event::Visibility {
                        window_id: window.id().clone(),
                        visible: true,
                    });
                }
            },

            xcb_sys::XCB_UNMAP_NOTIFY => {
                let ev = unsafe { *(xevent_ptr as *const xcb_sys::xcb_unmap_notify_event_t) };
                if let Some(window) = self.window_manager.borrow().get(ev.window).cloned() {
                    window.update_visibility(false);
                    f(Event::Visibility {
                        window_id: window.id().clone(),
                        visible: false,
                    });
                }
            },

            _ => (),
        }

        Ok(())
    }
}

impl<W: 'static + Clone> Context<W> {
    /// Performs any initialization on the context that occurs after the connection is obtained.
    fn init(connection: Connection) -> Result<Context<W>> {
        #[cfg(feature = "x11-sys")]
        unsafe {
            x11_sys::XSetEventQueueOwner(
                connection.xlib,
                x11_sys::XEventQueueOwner_XCBOwnsEventQueue,
            );
        }

        let atoms = Atoms::intern(&connection)?;
        let xcb = connection.xcb;

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
            if device.screen_index() == self.connection.default_screen_index {
                return device;
            }
        }
        panic!("invalid default X screen");
    }

    fn devices(&self) -> Devices<W> {
        Devices::new(&self)
    }

    fn run<F: FnMut(Event<Self::WindowId>)>(&self, main_loop: &MainLoop, mut f: F) -> Result<()> {
        main_loop.clear_quit_flag();

        'main_loop: while !main_loop.is_quit_requested() {
            self.flush()?;

            unsafe {
                let mut xevent_ptr = xcb_sys::xcb_poll_for_event(self.xcb);
                if xevent_ptr.is_null() {
                    match main_loop.update_kind() {
                        UpdateKind::Passive => {
                            f(Event::Update {
                                kind: UpdateKind::Passive,
                            });
                            if main_loop.is_quit_requested() {
                                break 'main_loop;
                            }
                            self.flush()?;
                            xevent_ptr = xcb_sys::xcb_wait_for_event(self.xcb);
                        },

                        UpdateKind::Active | UpdateKind::VBlank => {
                            f(Event::Update {
                                kind: UpdateKind::Active,
                            });
                            continue 'main_loop;
                        },
                    }
                }
                if !xevent_ptr.is_null() {
                    let xevent = CBox::from_raw(xevent_ptr);
                    self.handle_event(xevent, &mut f)?;
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
