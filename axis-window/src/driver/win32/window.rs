/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::any::TypeId;
use std::cell::Cell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use math::Vector2;

use crate::driver::win32::context::{Context, EventManager};
use crate::driver::win32::device::Device;
use crate::driver::win32::gdi::Dc;
use crate::driver::win32::pixel_format::PixelFormat;
use crate::driver::win32::util::{get_exe_handle, Win32Error};
use crate::error::{ErrorKind, Result};
use crate::event::Event;
use crate::window::{IWindow, IWindowBuilder, WindowKind, WindowPos};
use crate::Coord;

/// Win32 window builder type.
pub struct WindowBuilder<W: 'static + Clone> {
    event_manager: Rc<EventManager<W>>,
    kind: WindowKind,
    _phantom: PhantomData<W>,
    pixel_format: PixelFormat,
    pos: WindowPos,
    size: Option<Vector2<Coord>>,
    title: String,
}

impl<W: 'static + Clone> WindowBuilder<W> {
    pub(crate) fn new(device: &Device<W>) -> WindowBuilder<W> {
        WindowBuilder {
            event_manager: device.event_manager().clone(),
            kind: WindowKind::Normal,
            _phantom: PhantomData,
            pixel_format: PixelFormat::Default,
            pos: WindowPos::Default,
            size: None,
            title: String::new(),
        }
    }
}

impl<W: 'static + Clone> IWindowBuilder for WindowBuilder<W> {
    type Context = Context<W>;

    fn build(&self, id: W) -> Result<Window<W>> {
        let class_name_ptr = WINDOW_CLASS_MANAGER.lock()?.register::<W>()?;
        let title: Vec<u16> = self
            .title
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let (style, ex_style) = match self.kind {
            WindowKind::Normal => (winapi::um::winuser::WS_OVERLAPPEDWINDOW, 0),
        };
        let pos = match self.pos {
            WindowPos::Default => Vector2::new(
                winapi::um::winuser::CW_USEDEFAULT,
                winapi::um::winuser::CW_USEDEFAULT,
            ),
            WindowPos::Centered => {
                // TODO
                Vector2::new(
                    winapi::um::winuser::CW_USEDEFAULT,
                    winapi::um::winuser::CW_USEDEFAULT,
                )
            },
            WindowPos::Point(pos) => pos,
        };
        let hinstance = get_exe_handle()?;

        // If size is specified, interpret it is client size, not full window size.
        let size = match self.size {
            None => Vector2 {
                x: winapi::um::winuser::CW_USEDEFAULT,
                y: winapi::um::winuser::CW_USEDEFAULT,
            },
            Some(size) => {
                let mut rect = winapi::shared::windef::RECT {
                    left: 0,
                    top: 0,
                    right: size.x,
                    bottom: size.y,
                };

                unsafe {
                    if winapi::um::winuser::AdjustWindowRectEx(&mut rect, style, 0, ex_style) == 0 {
                        return Err(err!(SystemError("AdjustWindowRectEx"): Win32Error::last()));
                    }
                }

                Vector2::new(rect.right - rect.left, rect.bottom - rect.top)
            },
        };

        let hwnd;

        unsafe {
            hwnd = winapi::um::winuser::CreateWindowExW(
                ex_style,
                class_name_ptr,
                title.as_ptr(),
                style,
                pos.x,
                pos.y,
                size.x,
                size.y,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                hinstance,
                std::ptr::null_mut(),
            );
        }

        if hwnd.is_null() {
            return Err(err!(SystemError("CreateWindowExW"): Win32Error::last()));
        }

        let window = Window {
            shared: Rc::new(WindowShared {
                event_manager: self.event_manager.clone(),
                hwnd: Cell::new(Some(hwnd)),
                id,
            }),
        };

        window.shared.set_pixel_format(&self.pixel_format)?;

        // So we can properly report events.
        window.shared.set_window_long_ptr(
            winapi::um::winuser::GWLP_USERDATA,
            Rc::into_raw(window.shared.clone()) as isize,
        )?;
        Ok(window)
    }
}

/// Data shared between a [`Window`] and its underlying `HWND`.
pub struct WindowShared<W: 'static + Clone> {
    event_manager: Rc<EventManager<W>>,
    hwnd: Cell<Option<winapi::shared::windef::HWND>>,
    id: W,
}

impl<W: 'static + Clone> WindowShared<W> {
    pub fn try_hwnd(&self) -> Result<winapi::shared::windef::HWND> {
        match self.hwnd.get() {
            None => Err(err!(ResourceExpired("window destroyed"))),
            Some(hwnd) => Ok(hwnd),
        }
    }
}

impl<W: 'static + Clone> WindowShared<W> {
    fn destroy(&self) {
        if let Some(hwnd) = self.hwnd.take() {
            unsafe {
                if winapi::um::winuser::DestroyWindow(hwnd) == 0 {
                    error!("DestroyWindow failed: {}", Win32Error::last());
                }
            }
        }
    }

    unsafe fn from_hwnd<'a>(hwnd: winapi::shared::windef::HWND) -> Option<&'a WindowShared<W>> {
        if hwnd.is_null() {
            return None;
        }
        winapi::um::errhandlingapi::SetLastError(0);
        let value =
            winapi::um::winuser::GetWindowLongPtrW(hwnd, winapi::um::winuser::GWLP_USERDATA);
        if let Some(err) = Win32Error::try_last() {
            error!("GetWindowLongPtrW: {}", err);
            return None;
        } else if value == 0 {
            return None;
        }
        Some(&*(value as *const WindowShared<W>))
    }

    fn get_window_long(&self, index: i32) -> Result<i32> {
        unsafe {
            let hwnd = self.try_hwnd()?;
            winapi::um::errhandlingapi::SetLastError(0);
            let value = winapi::um::winuser::GetWindowLongW(hwnd, index);
            if let Some(err) = Win32Error::try_last() {
                return Err(err!(SystemError("GetWindowLongW"): err));
            }
            Ok(value)
        }
    }

    fn set_pixel_format(self: &Rc<Self>, pixel_format: &PixelFormat) -> Result<()> {
        match *pixel_format {
            PixelFormat::Default => (),
            PixelFormat::Gdi { index, pfd } => {
                let dc = Dc::get(self)?;
                let pfd_size = std::mem::size_of::<winapi::um::wingdi::PIXELFORMATDESCRIPTOR>();

                unsafe {
                    // Before we change the pixel format, let's query the PFD at the specified index
                    // and make sure it matches what was provided.
                    let mut pfd2: winapi::um::wingdi::PIXELFORMATDESCRIPTOR = std::mem::zeroed();
                    let result = winapi::um::wingdi::DescribePixelFormat(
                        dc.hdc(),
                        index,
                        pfd_size as u32,
                        &mut pfd2,
                    );
                    if result == 0 {
                        return Err(err!(SystemError("DescribePixelFormat"): Win32Error::last()));
                    }
                    let pixel_format_2 = PixelFormat::Gdi { index, pfd: pfd2 };
                    if *pixel_format != pixel_format_2 {
                        return Err(err!(InvalidArgument("pixel format mismatch")));
                    }

                    // Now we can change the pixel format.
                    if winapi::um::wingdi::SetPixelFormat(dc.hdc(), index, &pfd) == 0 {
                        return Err(err!(SystemError("SetPixelFormat"): Win32Error::last()));
                    }
                }
            },
        }
        Ok(())
    }

    fn set_window_long_ptr(&self, index: i32, value: isize) -> Result<isize> {
        unsafe {
            let hwnd = self.try_hwnd()?;
            winapi::um::errhandlingapi::SetLastError(0);
            let prev = winapi::um::winuser::SetWindowLongPtrW(hwnd, index, value);
            if let Some(err) = Win32Error::try_last() {
                return Err(err!(SystemError("SetWindowLongPtrW"): err));
            }
            Ok(prev)
        }
    }
}

impl<W: 'static + Clone> Drop for WindowShared<W> {
    fn drop(&mut self) {
        self.destroy();
    }
}

/// Win32 top-level window type.
pub struct Window<W: 'static + Clone> {
    shared: Rc<WindowShared<W>>,
}

impl<W: 'static + Clone> Drop for Window<W> {
    fn drop(&mut self) {
        self.shared.destroy();
    }
}

impl<W: 'static + Clone> IWindow for Window<W> {
    type Context = Context<W>;

    fn id(&self) -> &W {
        &self.shared.id
    }
    fn is_alive(&self) -> bool {
        self.shared.hwnd.get().is_some()
    }

    fn is_visible(&self) -> bool {
        let style = match self.shared.get_window_long(winapi::um::winuser::GWL_STYLE) {
            Ok(value) => value as u32,
            Err(err) => {
                if err.kind() != ErrorKind::ResourceExpired {
                    error!("{}", err);
                }
                return false;
            },
        };
        style & winapi::um::winuser::WS_VISIBLE != 0
    }

    fn set_visible(&mut self, visible: bool) -> Result<()> {
        let hwnd = self.shared.try_hwnd()?;
        let command = match visible {
            false => winapi::um::winuser::SW_HIDE,
            true => winapi::um::winuser::SW_SHOW,
        };

        unsafe {
            winapi::um::winuser::ShowWindow(hwnd, command);
        }

        Ok(())
    }
}

/// Registers the Win32 window class.
struct WindowClassManager {
    map: HashMap<TypeId, Arc<Vec<u16>>>,
    next_num: u32,
}

lazy_static! {
    static ref WINDOW_CLASS_MANAGER: Arc<Mutex<WindowClassManager>> =
        Arc::new(Mutex::new(WindowClassManager {
            map: HashMap::new(),
            next_num: 0,
        }));
}

impl WindowClassManager {
    /// Registers a class (if it has not already been registered) and returns its null-terminated
    /// class name.
    ///
    /// If the name is already taken, this tries another name. This ensures that things don't break
    /// if multiple versions of this crate are used in a project.
    fn register<W: 'static + Clone>(&mut self) -> Result<*const u16> {
        let type_id = TypeId::of::<W>();
        if let Some(name) = self.map.get(&type_id) {
            return Ok(name.as_ptr());
        }
        let hinstance = get_exe_handle()?;
        let hcursor;

        unsafe {
            hcursor = winapi::um::winuser::LoadCursorW(
                std::ptr::null_mut(),
                winapi::um::winuser::IDC_ARROW,
            );
        }

        if hcursor.is_null() {
            return Err(err!(SystemError("LoadCursorW"): Win32Error::last()));
        }

        let mut wc = winapi::um::winuser::WNDCLASSEXW {
            cbSize: std::mem::size_of::<winapi::um::winuser::WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(wndproc::<W>),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: std::ptr::null_mut(),
            hCursor: hcursor,
            hbrBackground: std::ptr::null_mut(),
            lpszMenuName: std::ptr::null(),
            lpszClassName: std::ptr::null(),
            hIconSm: std::ptr::null_mut(),
        };

        let mut name: Vec<u16>;

        'name_loop: loop {
            name = format!("AxisWindow_{}", self.next_num)
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            wc.lpszClassName = name.as_ptr();

            unsafe {
                if winapi::um::winuser::RegisterClassExW(&wc) == 0 {
                    let err = Win32Error::last();
                    match err.0 {
                        winapi::shared::winerror::ERROR_CLASS_ALREADY_EXISTS => {
                            self.next_num += 1;
                            continue 'name_loop;
                        },
                        _ => return Err(err!(SystemError("RegisterClassExW"): err)),
                    }
                }
            }

            break 'name_loop;
        }

        let name_ptr = name.as_ptr();
        self.map.insert(type_id, Arc::new(name));
        self.next_num += 1;
        Ok(name_ptr)
    }
}

/// Window message handler.
unsafe extern "system" fn wndproc<W: 'static + Clone>(
    hwnd: winapi::shared::windef::HWND, msg: u32, wparam: usize, lparam: isize,
) -> isize {
    match msg {
        winapi::um::winuser::WM_CLOSE => {
            if let Some(window) = WindowShared::<W>::from_hwnd(hwnd) {
                window.event_manager.push(Event::Close {
                    window_id: window.id.clone(),
                });
            }
            0
        },

        winapi::um::winuser::WM_DESTROY => {
            if let Some(window) = WindowShared::<W>::from_hwnd(hwnd) {
                window.hwnd.set(None);
                let result = window.set_window_long_ptr(winapi::um::winuser::GWLP_USERDATA, 0);
                if let Ok(shared_ptr) = result {
                    let _ = Rc::from_raw(shared_ptr as *const WindowShared<W>);
                }
                window.event_manager.push(Event::Destroy {
                    window_id: window.id.clone(),
                });
            }
            0
        },

        winapi::um::winuser::WM_SHOWWINDOW => {
            if let Some(window) = WindowShared::<W>::from_hwnd(hwnd) {
                window.event_manager.push(Event::Visibility {
                    visible: wparam != 0,
                    window_id: window.id.clone(),
                });
            }
            0
        },

        _ => winapi::um::winuser::DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
