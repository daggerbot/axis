/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::cell::Cell;
use std::ffi::CString;
use std::marker::PhantomData;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd};
use std::os::raw::c_char;
use std::rc::Rc;

use crate::client::IClient;
use crate::driver::x11::pixel_format::PixelFormat;
use crate::driver::x11::window::{
    ChangePropertyMode,
    PropertyData,
    Window,
    WindowBuilder,
    WindowManager,
};
use crate::error::Result;
use crate::event::{Event, MainLoop, UpdateMode};

/// Connection to an X11 display server.
pub struct Connection {
    #[cfg(feature = "x11-sys")]
    xlib: *mut x11_sys::Display,
    xcb: *mut xcb_sys::xcb_connection_t,
    default_screen_num: u8,
}

impl Connection {
    /// Returns the default screen number.
    pub fn default_screen_num(&self) -> u8 {
        self.default_screen_num
    }

    /// Connects to the specified X11 display server.
    pub fn open<S: Into<Vec<u8>>>(name: S) -> Result<Connection> {
        unsafe {
            Connection::open_raw(CString::new(name)?.as_ptr())
        }
    }

    /// Connects to the default X11 display server.
    pub fn open_default() -> Result<Connection> {
        unsafe {
            Connection::open_raw(std::ptr::null())
        }
    }

    /// Connects to an X11 display server specified by a raw C string.
    pub unsafe fn open_raw(name_ptr: *const c_char) -> Result<Connection> {
        #[cfg(feature = "x11-sys")]
        let xlib;
        let xcb;
        #[allow(unused_assignments)]
        let mut default_screen_num = 0;

        #[cfg(feature = "x11-sys")]
        {
            xlib = x11_sys::XOpenDisplay(name_ptr);
            if xlib.is_null() {
                return Err(err!(ConnectionFailed("XOpenDisplay failed")));
            }

            x11_sys::XSetEventQueueOwner(xlib, x11_sys::XEventQueueOwner_XCBOwnsEventQueue);
            xcb = x11_sys::XGetXCBConnection(xlib) as *mut xcb_sys::xcb_connection_t;
            if xcb.is_null() {
                return Err(err!(ConnectionFailed("XGetXCBConnection failed")));
            }

            default_screen_num = x11_sys::XDefaultScreen(xlib);
        }

        #[cfg(not(feature = "x11-sys"))]
        {
            xcb = xcb_sys::xcb_connect(name_ptr, &mut default_screen_num);
            if xcb.is_null() {
                return Err(err!(ConnectionFailed("xcb_connect failed")));
            }
        }

        Ok(Connection {
            #[cfg(feature = "x11-sys")]
            xlib,
            xcb,
            default_screen_num: match u8::try_from(default_screen_num) {
                Ok(n) => n,
                Err(err) => return Err(err!(ConnectionFailed("invalid default X screen"): err)),
            },
        })
    }

    /// Returns the underlying XCB connection handle.
    pub fn xcb_connection_ptr(&self) -> *mut xcb_sys::xcb_connection_t {
        self.xcb
    }

    /// Returns the underlying Xlib display handle.
    #[cfg(feature = "x11-sys")]
    pub fn xlib_display_ptr(&self) -> *mut x11_sys::Display {
        self.xlib
    }
}

impl Connection {
    pub(crate) fn change_property<T: ?Sized + PropertyData>(
        &self, mode: ChangePropertyMode, window: u32, property: u32, ty: u32, data: &T)
        -> xcb_sys::xcb_void_cookie_t
    {
        unsafe {
            xcb_sys::xcb_change_property(self.xcb, mode as u8, window, property, ty, T::format(),
                                         data.len(), data.as_ptr())
        }
    }

    pub(crate) fn intern_atom(&self, name: &str) -> xcb_sys::xcb_intern_atom_cookie_t {
        unsafe {
            xcb_sys::xcb_intern_atom(self.xcb, 0, name.len() as u16, name.as_ptr() as *const c_char)
        }
    }

    pub(crate) fn intern_atom_reply(&self, cookie: xcb_sys::xcb_intern_atom_cookie_t)
        -> Result<u32>
    {
        unsafe {
            let mut err_ptr = std::ptr::null_mut();
            let reply_ptr = xcb_sys::xcb_intern_atom_reply(self.xcb, cookie, &mut err_ptr);

            if reply_ptr.is_null() {
                if err_ptr.is_null() {
                    return Err(err!(RequestFailed("X_InternAtom")));
                } else {
                    let err = err!(RequestFailed{"X_InternAtom: {:?}", *err_ptr});
                    libc::free(err_ptr as *mut _);
                    return Err(err);
                }
            }

            let atom = (*reply_ptr).atom;
            libc::free(reply_ptr as *mut _);
            if !err_ptr.is_null() {
                libc::free(err_ptr as *mut _);
            }
            Ok(atom)
        }
    }
}

impl AsFd for Connection {
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe {
            BorrowedFd::borrow_raw(self.as_raw_fd())
        }
    }
}

impl AsRawFd for Connection {
    fn as_raw_fd(&self) -> RawFd {
        unsafe {
            xcb_sys::xcb_get_file_descriptor(self.xcb)
        }
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        unsafe {
            #[cfg(feature = "x11-sys")]
            x11_sys::XCloseDisplay(self.xlib);

            #[cfg(not(feature = "x11-sys"))]
            xcb_sys::xcb_disconnect(self.xcb);
        }
    }
}

impl Eq for Connection {}

impl PartialEq for Connection {
    fn eq(&self, rhs: &Connection) -> bool {
        self as *const _ == rhs as *const _
    }
}

/// X11 window system client type.
pub struct Client<W: 'static + Clone> {
    atoms: Rc<Atoms>,
    connection: Rc<Connection>,
    _phantom: PhantomData<W>,
    screens: Rc<Vec<Screen>>,
    window_manager: Rc<WindowManager<W>>,
}

impl<W: 'static + Clone> Client<W> {
    /// Returns the underlying connection.
    pub fn connection(&self) -> &Rc<Connection> {
        &self.connection
    }

    /// Returns the default X11 screen.
    pub fn default_screen(&self) -> Screen {
        self.screens[self.connection.default_screen_num as usize].clone()
    }

    /// Connects to the specified X11 display server.
    pub fn open<S: Into<Vec<u8>>>(name: S) -> Result<Client<W>> {
        Client::init(Connection::open(name)?)
    }

    /// Connects to the default X11 display server.
    pub fn open_default() -> Result<Client<W>> {
        Client::init(Connection::open_default()?)
    }

    /// Connects to an X11 display server specified by a raw C string.
    pub unsafe fn open_raw(name_ptr: *const c_char) -> Result<Client<W>> {
        Client::init(Connection::open_raw(name_ptr)?)
    }

    /// Gets an iterator of all available X11 screens.
    pub fn screens(&self) -> impl Iterator<Item = Screen> {
        (*self.screens).clone().into_iter()
    }
}

impl<W: 'static + Clone> Client<W> {
    pub(crate) fn atoms(&self) -> &Rc<Atoms> {
        &self.atoms
    }

    pub(crate) fn screens_ref(&self) -> &Rc<Vec<Screen>> {
        &self.screens
    }

    pub(crate) fn window_manager(&self) -> &Rc<WindowManager<W>> {
        &self.window_manager
    }
}

impl<W: 'static + Clone> Client<W> {
    fn check_connection(&self) -> Result<()> {
        let result;

        unsafe {
            result = xcb_sys::xcb_connection_has_error(self.connection.xcb);
        }

        Err(err!(IoError(match result as u32 {
            0 => return Ok(()),
            xcb_sys::XCB_CONN_ERROR => "X11 connection error",
            xcb_sys::XCB_CONN_CLOSED_EXT_NOTSUPPORTED => "X11 extension not supported",
            xcb_sys::XCB_CONN_CLOSED_MEM_INSUFFICIENT => "memory insufficient",
            xcb_sys::XCB_CONN_CLOSED_REQ_LEN_EXCEED => "X11 request length exceeded",
            xcb_sys::XCB_CONN_CLOSED_PARSE_ERR => "X11 display name parse error",
            xcb_sys::XCB_CONN_CLOSED_INVALID_SCREEN => "invalid X11 screen",
            _ => return Err(err!(IoError)),
        })))
    }

    unsafe fn handle_x_event<F: Fn(Event<W>)>(
        &self, event: *const xcb_sys::xcb_generic_event_t, f: &F) -> Result<()>
    {
        match ((*event).response_type & !0x80) as u32 {
            xcb_sys::XCB_CLIENT_MESSAGE => {
                let ev = event as *const xcb_sys::xcb_client_message_event_t;
                if let Some(window) = self.window_manager.get((*ev).window) {
                    if (*ev).type_ == self.atoms.WM_PROTOCOLS && (*ev).format == 32 {
                        let protocol = (*ev).data.data32[0];
                        if protocol == self.atoms.WM_DELETE_WINDOW {
                            f(Event::CloseRequest {
                                window_id: window.id().clone(),
                            });
                        }
                    }
                }
            },

            xcb_sys::XCB_DESTROY_NOTIFY => {
                let ev = event as *const xcb_sys::xcb_destroy_notify_event_t;
                if let Some(window) = self.window_manager.unregister((*ev).window) {
                    f(Event::Destroy {
                        window_id: window.id().clone(),
                    });
                }
            },

            xcb_sys::XCB_MAP_NOTIFY => {
                let ev = event as *const xcb_sys::xcb_map_notify_event_t;
                if let Some(window) = self.window_manager.get((*ev).window) {
                    if let Some(event) = window.update_visibility(true) {
                        f(event);
                    }
                }
            },

            xcb_sys::XCB_UNMAP_NOTIFY => {
                let ev = event as *const xcb_sys::xcb_unmap_notify_event_t;
                if let Some(window) = self.window_manager.get((*ev).window) {
                    if let Some(event) = window.update_visibility(false) {
                        f(event);
                    }
                }
            },

            _ => (),
        }

        Ok(())
    }

    /// Initializes a client from a connection.
    fn init(connection: Connection) -> Result<Client<W>> {
        let connection = Rc::new(connection);
        let atoms = Rc::new(Atoms::init(connection.as_ref())?);
        let mut screens = Vec::new();

        unsafe {
            let setup_ptr = xcb_sys::xcb_get_setup(connection.xcb);
            if setup_ptr.is_null() {
                return Err(err!(RuntimeError("xcb_get_setup returned null")));
            }

            let mut screen_iter = xcb_sys::xcb_setup_roots_iterator(setup_ptr);
            let mut screen_num = 0;
            while screen_iter.rem > 0 {
                let screen_ptr = screen_iter.data;
                screens.push(Screen::new(&connection, screen_num, screen_ptr));
                screen_num += 1;
                xcb_sys::xcb_screen_next(&mut screen_iter);
            }
        }

        Ok(Client {
            atoms,
            connection: connection,
            _phantom: PhantomData,
            screens: Rc::new(screens),
            window_manager: Rc::new(WindowManager::new()),
        })
    }
}

impl<W: 'static + Clone> IClient for Client<W> {
    type PixelFormat = PixelFormat;
    type Window = Window<W>;
    type WindowBuilder = WindowBuilder<W>;
    type WindowId = W;

    fn default_pixel_format(&self) -> PixelFormat {
        self.default_screen().default_pixel_format()
    }

    fn run<F: Fn(Event<W>)>(&self, main_loop: &MainLoop, f: &F) -> Result<()> {
        let need_update = Cell::new(true);
        let f = |event| {
            match event {
                Event::Update { .. } => (),
                _ => need_update.set(true),
            }
            f(event);
        };

        'main_loop: while !main_loop.is_quit_requested() {
            unsafe {
                xcb_sys::xcb_flush(self.connection.xcb);
                self.check_connection()?;

                // Handle pending events.
                'poll_loop: loop {
                    let event_ptr = xcb_sys::xcb_poll_for_event(self.connection.xcb);
                    if event_ptr.is_null() {
                        break 'poll_loop;
                    }
                    self.handle_x_event(event_ptr, &f)?;
                    libc::free(event_ptr as *mut _);
                    if main_loop.is_quit_requested() {
                        break 'main_loop;
                    }
                }

                // Emit update event and possibly wait for more events.
                match main_loop.update_mode() {
                    UpdateMode::Passive => {
                        if need_update.take() {
                            f(Event::Update { update_mode: UpdateMode::Passive });
                            if main_loop.is_quit_requested() {
                                break 'main_loop;
                            }
                        }

                        let event_ptr = xcb_sys::xcb_wait_for_event(self.connection.xcb);
                        if event_ptr.is_null() {
                            self.check_connection()?;
                            return Err(err!(IoError));
                        }
                        self.handle_x_event(event_ptr, &f)?;
                        libc::free(event_ptr as *mut _);
                    },

                    UpdateMode::Active | UpdateMode::Sync => {
                        f(Event::Update { update_mode: UpdateMode::Active });
                    },
                }
            }
        }

        Ok(())
    }

    fn window(&self) -> WindowBuilder<W> {
        WindowBuilder::new(self)
    }
}

/// X11 screen type.
#[derive(Clone)]
pub struct Screen {
    connection: Rc<Connection>,
    num: u8,
    screen_ptr: *mut xcb_sys::xcb_screen_t,
}

impl Screen {
    /// Gets the underlying connection.
    pub fn connection(&self) -> &Rc<Connection> {
        &self.connection
    }

    /// Gets the default pixel format (visual).
    pub fn default_pixel_format(&self) -> PixelFormat {
        let visual_id;

        unsafe {
            visual_id = (*self.screen_ptr).root_visual;
        }

        for pf in self.pixel_formats() {
            if pf.visual_id() == visual_id {
                return pf;
            }
        }

        panic!("can't find X11 root visual");
    }

    /// Gets the screen number.
    pub fn num(&self) -> u8 {
        self.num
    }

    /// Gets an iterator over available pixel formats (visuals) for the screen.
    pub fn pixel_formats(&self) -> impl Iterator<Item = PixelFormat> {
        unsafe {
            let connection = self.connection.clone();
            let depth_iter = xcb_sys::xcb_screen_allowed_depths_iterator(self.screen_ptr);
            let visual_iter = match depth_iter.rem {
                0 => xcb_sys::xcb_visualtype_iterator_t {
                    data: std::ptr::null_mut(),
                    index: 0,
                    rem: 0,
                },
                _ => xcb_sys::xcb_depth_visuals_iterator(depth_iter.data),
            };

            PixelFormats {
                connection,
                depth_iter,
                screen_num: self.num,
                visual_iter,
            }
        }
    }

    /// Gets the resource ID of the screen's root window.
    pub fn root(&self) -> u32 {
        unsafe {
            (*self.screen_ptr).root
        }
    }

    /// Gets the XCB screen struct pointer.
    pub fn xcb_screen_ptr(&self) -> *mut xcb_sys::xcb_screen_t {
        self.screen_ptr
    }
}

impl Screen {
    unsafe fn new(connection: &Rc<Connection>, num: u8, screen_ptr: *mut xcb_sys::xcb_screen_t)
        -> Screen
    {
        Screen {
            connection: connection.clone(),
            num,
            screen_ptr,
        }
    }
}

impl Eq for Screen {}

impl PartialEq for Screen {
    fn eq(&self, rhs: &Screen) -> bool {
        self.screen_ptr == rhs.screen_ptr
    }
}

/// Iterator over X11 visuals available for a screen.
struct PixelFormats {
    connection: Rc<Connection>,
    depth_iter: xcb_sys::xcb_depth_iterator_t,
    screen_num: u8,
    visual_iter: xcb_sys::xcb_visualtype_iterator_t,
}

impl Iterator for PixelFormats {
    type Item = PixelFormat;

    fn next(&mut self) -> Option<PixelFormat> {
        unsafe {
            while self.visual_iter.rem == 0 {
                match self.depth_iter.rem {
                    0 => return None,
                    _ => xcb_sys::xcb_depth_next(&mut self.depth_iter),
                }
            }

            let next = PixelFormat::new(&self.connection, self.screen_num,
                                        (*self.depth_iter.data).depth, self.visual_iter.data);
            xcb_sys::xcb_visualtype_next(&mut self.visual_iter);
            Some(next)
        }
    }
}

/// Defines the `Atoms` type.
macro_rules! define_atoms {
    { $($name:ident,)* } => {
        #[allow(non_snake_case)]
        pub struct Atoms {
            $(pub $name: u32),*
        }

        impl Atoms {
            #[allow(non_snake_case)]
            fn init(connection: &Connection) -> Result<Atoms> {
                let mut cookies = Vec::new();
                $(cookies.push(connection.intern_atom(stringify!($name)));)*
                let mut cookies = cookies.into_iter();
                $(let $name = connection.intern_atom_reply(cookies.next().unwrap())?;)*
                Ok(Atoms { $($name),* })
            }
        }
    };
}

define_atoms! {
    WM_DELETE_WINDOW,
    WM_PROTOCOLS,
}
