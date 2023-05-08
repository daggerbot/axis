/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::any::TypeId;
use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use vectorial::Vec2;
use winapi::shared::windef::HWND;
use winapi::um::winuser::WNDCLASSEXW;

use crate::driver::win32::client::{Client, EventManager};
use crate::error::Result;
use crate::event::Event;
use crate::ffi;
use crate::window::{IWindow, IWindowBuilder};
use crate::Coord;

/// Win32 window builder.
pub struct WindowBuilder<W: 'static + Clone> {
    class_name: Rc<Vec<u16>>,
    event_manager: Rc<EventManager<W>>,
    pos: Option<Vec2<Coord>>,
    size: Option<Vec2<Coord>>,
    title: String,
}

impl<W: 'static + Clone> WindowBuilder<W> {
    /// Constructs a window builder.
    pub(crate) fn new(client: &Client<W>) -> WindowBuilder<W> {
        WindowBuilder {
            class_name: client.window_class_name().clone(),
            event_manager: client.event_manager().clone(),
            pos: None,
            size: None,
            title: String::new(),
        }
    }
}

impl<W: 'static + Clone> IWindowBuilder for WindowBuilder<W> {
    type Client = Client<W>;

    fn build(&self, id: W) -> Result<Window<W>> {
        Window::new(self, id)
    }
}

/// Data shared between an `HWND` and a [Window].
struct WindowData<W: 'static + Clone> {
    event_manager: Rc<EventManager<W>>,
    hwnd: Cell<HWND>,
    id: W,
}

impl<W: 'static + Clone> WindowData<W> {
    /// Gets a `WindowData` from a `HWND`.
    unsafe fn get<'a>(hwnd: HWND) -> Option<&'a WindowData<W>> {
        match winapi::um::winuser::GetWindowLongPtrW(hwnd, winapi::um::winuser::GWLP_USERDATA) {
            0 => None,
            data => Some(&*(data as *const WindowData<W>)),
        }
    }

    /// Takes the window data from a `HWND`'s `GWLP_USERDATA` field.
    unsafe fn take(hwnd: HWND) -> Option<Rc<WindowData<W>>> {
        match winapi::um::winuser::GetWindowLongPtrW(hwnd, winapi::um::winuser::GWLP_USERDATA) {
            0 => None,
            data => {
                winapi::um::winuser::SetWindowLongPtrW(hwnd, winapi::um::winuser::GWLP_USERDATA, 0);
                Some(Rc::from_raw(data as *const WindowData<W>))
            },
        }
    }
}

/// Win32 window type.
pub struct Window<W: 'static + Clone> {
    data: Rc<WindowData<W>>,
}

impl<W: 'static + Clone> Window<W> {
    /// Returns the underlying window handle.
    pub fn hwnd(&self) -> HWND {
        self.data.hwnd.get()
    }

    /// Returns the underlying window handle, or an error if the window is expired.
    pub fn try_hwnd(&self) -> Result<HWND> {
        let hwnd = self.hwnd();
        if hwnd.is_null() {
            return Err(err!(ResourceExpired("window expired")));
        }
        Ok(hwnd)
    }
}

impl<W: 'static + Clone> Window<W> {
    fn get_style(&self) -> Result<u32> {
        Ok(self.get_window_long(winapi::um::winuser::GWL_STYLE)? as u32)
    }

    fn get_window_long(&self, index: i32) -> Result<i32> {
        unsafe {
            winapi::um::errhandlingapi::SetLastError(0);
            let value = winapi::um::winuser::GetWindowLongW(self.try_hwnd()?, index);
            if let Some(err) = ffi::win32::Error::get() {
                return Err(err!(RuntimeError("GetWindowLongW"): err));
            }
            Ok(value)
        }
    }

    fn new(builder: &WindowBuilder<W>, id: W) -> Result<Window<W>> {
        let style = winapi::um::winuser::WS_OVERLAPPEDWINDOW;
        let ex_style = 0;
        let class_name = builder.class_name.as_ptr();
        let title: Vec<u16> = builder.title.encode_utf16().chain(std::iter::repeat(0).take(1))
                              .collect();
        let pos = match builder.pos {
            None => Vec2::new(winapi::um::winuser::CW_USEDEFAULT,
                              winapi::um::winuser::CW_USEDEFAULT),
            Some(pos) => pos,
        };
        let size = match builder.size {
            None => Vec2::new(winapi::um::winuser::CW_USEDEFAULT,
                              winapi::um::winuser::CW_USEDEFAULT),
            Some(size) => Vec2::new(std::cmp::max(size.x, 1), std::cmp::max(size.y, 1)),
        };
        let hinstance = ffi::win32::get_exe_handle()?;
        let hwnd;

        unsafe {
            hwnd = winapi::um::winuser::CreateWindowExW(ex_style, class_name, title.as_ptr(),
                                                        style, pos.x, pos.y, size.x, size.y,
                                                        std::ptr::null_mut(), std::ptr::null_mut(),
                                                        hinstance, std::ptr::null_mut());
        }

        if hwnd.is_null() {
            return Err(err!(RuntimeError("CreateWindowExW"): ??w));
        }

        let data = Rc::new(WindowData {
            event_manager: builder.event_manager.clone(),
            hwnd: Cell::new(hwnd),
            id,
        });

        unsafe {
            let data_ptr = Rc::into_raw(data.clone());
            winapi::um::errhandlingapi::SetLastError(0);
            winapi::um::winuser::SetWindowLongPtrW(hwnd, winapi::um::winuser::GWLP_USERDATA,
                                                   data_ptr as isize);

            if let Some(err) = ffi::win32::Error::get() {
                let _ = Rc::from_raw(data_ptr);
                return Err(err!(RuntimeError("SetWindowLongPtrW"): err));
            }
        }

        Ok(Window {
            data,
        })
    }
}

impl<W: 'static + Clone> IWindow for Window<W> {
    type Client = Client<W>;

    fn destroy(&self) {
        let hwnd = self.hwnd();
        if !hwnd.is_null() {
            unsafe {
                winapi::um::winuser::DestroyWindow(hwnd);
            }
        }
    }

    fn id(&self) -> &W {
        &self.data.id
    }

    fn is_visible(&self) -> bool {
        match self.get_style() {
            Ok(style) => style & winapi::um::winuser::WS_VISIBLE != 0,
            Err(_) => false,
        }
    }

    fn set_visible(&self, visible: bool) -> Result<()> {
        unsafe {
            if visible {
                let hwnd = self.try_hwnd()?;
                winapi::um::winuser::ShowWindow(hwnd, winapi::um::winuser::SW_SHOW);
            } else {
                let hwnd = self.hwnd();
                if !hwnd.is_null() {
                    winapi::um::winuser::ShowWindow(hwnd, winapi::um::winuser::SW_HIDE);
                }
            }
        }

        Ok(())
    }
}

/// Manages window classes.
pub struct WindowClassManager {
    map: HashMap<TypeId, u32>,
    next_id: u32,
}

impl WindowClassManager {
    /// Returns the global class manager.
    pub fn get() -> &'static Arc<Mutex<WindowClassManager>> {
        &CLASS_MANAGER
    }

    /// Registers the window class for `W` and returns its name.
    pub fn register<W: 'static + Clone>(&mut self) -> Result<Vec<u16>> {
        unsafe {
            let hcursor = winapi::um::winuser::LoadCursorW(std::ptr::null_mut(),
                                                           winapi::um::winuser::IDC_ARROW);
            if hcursor.is_null() {
                return Err(err!(RuntimeError("LoadCursorW"): ??w));
            }

            let mut wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: winapi::um::winuser::CS_OWNDC,
                lpfnWndProc: Some(window_proc::<W>),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: ffi::win32::get_exe_handle()?,
                hIcon: std::ptr::null_mut(),
                hCursor: hcursor,
                hbrBackground: std::ptr::null_mut(),
                lpszMenuName: std::ptr::null(),
                lpszClassName: std::ptr::null(),
                hIconSm: std::ptr::null_mut(),
            };

            loop {
                let name: Vec<u16> = format!("AxisWindow{}", self.next_id)
                    .encode_utf16()
                    .chain(std::iter::repeat(0).take(1))
                    .collect();
                wc.lpszClassName = name.as_ptr();

                match winapi::um::winuser::RegisterClassExW(&wc) {
                    0 => match ffi::win32::Error::get_code() {
                        winapi::shared::winerror::ERROR_CLASS_ALREADY_EXISTS => {
                            self.next_id += 1;
                        },
                        code => return Err(err!(RuntimeError("RegisterClassExW"):
                                                ?ffi::win32::Error::from_code(code))),
                    },
                    _ => {
                        self.map.insert(TypeId::of::<W>(), self.next_id);
                        self.next_id += 1;
                        return Ok(name);
                    },
                }
            }
        }
    }
}

lazy_static! {
    static ref CLASS_MANAGER: Arc<Mutex<WindowClassManager>> =
        Arc::new(Mutex::new(WindowClassManager {
            map: HashMap::new(),
            next_id: 0,
        }));
}

/// Window message handler.
unsafe extern "system" fn window_proc<W: 'static + Clone>(
    hwnd: HWND, msg: u32, wparam: usize, lparam: isize) -> isize
{
    match msg {
        winapi::um::winuser::WM_CLOSE => {
            if let Some(window) = WindowData::<W>::get(hwnd) {
                window.event_manager.push(Event::CloseRequest {
                    window_id: window.id.clone(),
                });
            }
            0
        },

        winapi::um::winuser::WM_DESTROY => {
            if let Some(window) = WindowData::<W>::take(hwnd) {
                window.hwnd.set(std::ptr::null_mut());
                window.event_manager.push(Event::Destroy {
                    window_id: window.id.clone(),
                });
            }
            0
        },

        winapi::um::winuser::WM_SHOWWINDOW => {
            if let Some(window) = WindowData::<W>::get(hwnd) {
                window.event_manager.push(Event::VisibilityChange {
                    window_id: window.id.clone(),
                    visible: wparam != 0,
                });
            }
            0
        },

        _ => winapi::um::winuser::DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
