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

use math::{IntoLossy, Vector2};

use crate::driver::win32::device::Device;
use crate::driver::win32::error::{Win32Error, clear_last_error};
use crate::driver::win32::event::EventManager;
use crate::driver::win32::ffi::get_exe_handle;
use crate::driver::win32::gdi::WindowDc;
use crate::driver::win32::pixel_format::PixelFormat;
use crate::driver::win32::system::System;
use crate::error::{ErrorKind, Result};
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
    visible: bool,
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
            visible: false,
        }
    }
}

impl<W: 'static + Clone> IWindowBuilder for WindowBuilder<W> {
    type System = System<W>;

    fn build(&self, id: W) -> Result<Window<W>> {
        let class_name_ptr = WINDOW_CLASS_MANAGER.lock()?.register::<W>()?;
        let title: Vec<u16> = self.title.encode_utf16().chain(std::iter::once(0)).collect();

        let (mut style, ex_style) = match self.kind {
            WindowKind::Normal => (winapi::um::winuser::WS_OVERLAPPEDWINDOW, 0),
        };
        if self.visible {
            style |= winapi::um::winuser::WS_VISIBLE;
        }

        let pos = match self.pos {
            WindowPos::Default => Vector2 {
                x: winapi::um::winuser::CW_USEDEFAULT,
                y: winapi::um::winuser::CW_USEDEFAULT,
            },
            WindowPos::Centered => {
                // TODO: Let's treat this the same as Default for the moment. There are some other
                // things we have to implement before we can center a window.
                Vector2::new(
                    winapi::um::winuser::CW_USEDEFAULT,
                    winapi::um::winuser::CW_USEDEFAULT,
                )
            },
            WindowPos::Point(pos) => pos,
        };

        // If size is specified, interpret it is client size, not full window size. This requires
        // using AdjustWindowRectEx().
        let size = match self.size {
            None => Vector2 {
                x: winapi::um::winuser::CW_USEDEFAULT,
                y: winapi::um::winuser::CW_USEDEFAULT,
            },

            Some(size) => {
                let mut rect = winapi::shared::windef::RECT {
                    left: 0,
                    top: 0,
                    right: std::cmp::max(size.x, 1),
                    bottom: std::cmp::max(size.y, 1),
                };

                unsafe {
                    if winapi::um::winuser::AdjustWindowRectEx(&mut rect, style, 0, ex_style) == 0 {
                        return Err(err!(SystemError("AdjustWindowRectEx"): Win32Error::last()));
                    }
                }

                Vector2::new(rect.right - rect.left, rect.bottom - rect.top)
            },
        };

        let hinstance = get_exe_handle()?;
        let hwnd;

        unsafe {
            hwnd = winapi::um::winuser::CreateWindowExW(ex_style, class_name_ptr, title.as_ptr(),
                                                        style, pos.x, pos.y, size.x, size.y,
                                                        std::ptr::null_mut(), std::ptr::null_mut(),
                                                        hinstance, std::ptr::null_mut());
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

        // We'll later use get_window_long_ptr() to get our WindowShared. We could use a hash table
        // like we do in other drivers, but this is more idiomatic Win32 and is potentially faster
        // if tons of events are handled.
        window.shared.set_window_long_ptr(
            winapi::um::winuser::GWLP_USERDATA,
            Rc::into_raw(window.shared.clone()) as isize,
        )?;

        Ok(window)
    }

    fn with_pos(&mut self, pos: WindowPos) -> &mut WindowBuilder<W> {
        self.pos = pos;
        self
    }

    fn with_size(&mut self, size: Option<Vector2<Coord>>) -> &mut WindowBuilder<W> {
        self.size = size;
        self
    }

    fn with_title<S: Into<String>>(&mut self, title: S) -> &mut WindowBuilder<W> {
        self.title = title.into();
        self
    }

    fn with_visibility(&mut self, visible: bool) -> &mut WindowBuilder<W> {
        self.visible = visible;
        self
    }
}

/// Data which is shared (via `Rc`) between our `Window` object and an `HWND`.
pub struct WindowShared<W: 'static + Clone> {
    event_manager: Rc<EventManager<W>>,
    hwnd: Cell<Option<winapi::shared::windef::HWND>>,
    id: W,
}

impl<W: 'static + Clone> WindowShared<W> {
    pub fn event_manager(&self) -> &Rc<EventManager<W>> {
        &self.event_manager
    }

    /// Removes the association between our `WindowShared` and the `HWND`.
    pub unsafe fn expire(hwnd: winapi::shared::windef::HWND) -> Option<Rc<WindowShared<W>>> {
        let window_ref = match WindowShared::<W>::from_hwnd(hwnd) {
            None => return None,
            Some(w) => w,
        };
        let window = Rc::from_raw(window_ref as *const WindowShared<W>);
        window.hwnd.set(None);
        winapi::um::winuser::SetWindowLongPtrW(hwnd, winapi::um::winuser::GWLP_USERDATA, 0);
        Some(window)
    }

    /// Gets a `WindowShared` from an `HWND`'s `GWLP_USERDATA` value. This is unsafe because the
    /// Win32 API allows this value to be anything, not just a valid pointer. Use this only if it
    /// can be assumed that we created the `HWND` passed here.
    pub unsafe fn from_hwnd<'a>(hwnd: winapi::shared::windef::HWND) -> Option<&'a WindowShared<W>> {
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

    pub fn id(&self) -> &W {
        &self.id
    }

    /// Gets the underlying `HWND`, or returns an error if it is no longer valid.
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
                let dc = WindowDc::get(self)?;
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
            let prev = winapi::um::winuser::SetWindowLongPtrW(hwnd, index, value.into_lossy());
            if let Some(err) = Win32Error::try_last() {
                return Err(err!(SystemError("SetWindowLongPtrW"): err));
            }
            Ok(prev as isize)
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
    type System = System<W>;

    fn destroy(&self) {
        self.shared.destroy();
    }

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
                    error!("GetWindowLongPtrW: {}", err);
                }
                return false;
            },
        };
        style & winapi::um::winuser::WS_VISIBLE != 0
    }

    fn pos(&self) -> Result<Vector2<Coord>> {
        let hwnd = self.shared.try_hwnd()?;
        let mut rect;

        unsafe {
            rect = std::mem::zeroed();
            if winapi::um::winuser::GetWindowRect(hwnd, &mut rect) == 0 {
                return Err(err!(SystemError("GetWindowRect"): Win32Error::last()));
            }
        }

        Ok(Vector2 {
            x: Coord::from(rect.left),
            y: Coord::from(rect.top),
        })
    }

    fn set_pos(&self, pos: Vector2<Coord>) -> Result<()> {
        let hwnd = self.shared.try_hwnd()?;

        unsafe {
            if winapi::um::winuser::SetWindowPos(hwnd, std::ptr::null_mut(), pos.x, pos.y, 0, 0,
                                                 winapi::um::winuser::SWP_NOSIZE
                                                 | winapi::um::winuser::SWP_NOZORDER) == 0
            {
                return Err(err!(SystemError("SetWindowPos"): Win32Error::last()));
            }
        }

        Ok(())
    }

    fn set_size(&self, size: Vector2<Coord>) -> Result<()> {
        let hwnd = self.shared.try_hwnd()?;

        unsafe {
            if winapi::um::winuser::SetWindowPos(hwnd, std::ptr::null_mut(), 0, 0,
                                                 std::cmp::max(size.x, 1), std::cmp::max(size.y, 1),
                                                 winapi::um::winuser::SWP_NOMOVE
                                                 | winapi::um::winuser::SWP_NOZORDER) == 0
            {
                return Err(err!(SystemError("SetWindowPos"): Win32Error::last()));
            }
        }

        Ok(())
    }

    fn set_title(&self, title: &str) -> Result<()> {
        let hwnd = self.shared.try_hwnd()?;
        // NOTE: Should we return NulError if `title` contains any '\0'?
        let wtitle: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();

        unsafe {
            if winapi::um::winuser::SetWindowTextW(hwnd, wtitle.as_ptr()) == 0 {
                return Err(err!(SystemError("SetWindowTextW"): Win32Error::last()));
            }
        }

        Ok(())
    }

    fn set_visible(&self, visible: bool) -> Result<()> {
        let hwnd = self.shared.try_hwnd()?;
        let command = match visible {
            false => winapi::um::winuser::SW_HIDE,
            true => winapi::um::winuser::SW_SHOW,
        };

        unsafe {
            // The return value for ShowWindow doesn't indicate whether this failed. I've even
            // removed the GetLastError check because it seemed to be giving false positives.
            winapi::um::winuser::ShowWindow(hwnd, command);
        }

        Ok(())
    }

    fn size(&self) -> Result<Vector2<Coord>> {
        let hwnd = self.shared.try_hwnd()?;
        let mut rect;

        unsafe {
            rect = std::mem::zeroed();
            if winapi::um::winuser::GetClientRect(hwnd, &mut rect) == 0 {
                return Err(err!(SystemError("GetClientRect"): Win32Error::last()));
            }
        }

        Ok(Vector2 {
            x: Coord::from(rect.right - rect.left),
            y: Coord::from(rect.bottom - rect.top),
        })
    }

    fn title(&self) -> Result<String> {
        let hwnd = self.shared.try_hwnd()?;
        let mut buf: Vec<u16> = Vec::new();
        clear_last_error();

        unsafe {
            // Determine the length of the window title. The docs say that this value might be
            // greater than the actual length, but not less.
            let mut ilen = winapi::um::winuser::GetWindowTextLengthW(hwnd);
            if ilen <= 0 {
                match Win32Error::try_last() {
                    None => return Ok(String::new()),
                    Some(err) => return Err(err!(SystemError("GetWindowTextLengthW"): err)),
                }
            }
            let mut len = usize::try_from(ilen)?;
            buf.resize(len + 1, 0);

            // Get the actual window title. The length may not be what was previously indicated.
            ilen = winapi::um::winuser::GetWindowTextW(hwnd, buf.as_mut_ptr(), ilen + 1);
            if ilen <= 0 {
                match Win32Error::try_last() {
                    None => return Ok(String::new()),
                    Some(err) => return Err(err!(SystemError("GetWindowTextW"): err)),
                }
            }
            len = usize::try_from(ilen)?;
            buf.truncate(len);
        }

        Ok(String::from_utf16(buf.as_slice())?)
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
            lpfnWndProc: Some(crate::driver::win32::event::wndproc::<W>),
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

        // Loop until we successfully register our class. If multiple versions of this crate are
        // compiled into the executable, then multiple WindowClassManagers will exist, so we can't
        // rely on having a static name for this class. Increment the number at the end of the name
        // until this succeeds.
        'name_loop: loop {
            name = format!("AxisWindow_{}", self.next_num)
                   .encode_utf16().chain(std::iter::once(0)).collect();

            wc.lpszClassName = name.as_ptr();

            unsafe {
                if winapi::um::winuser::RegisterClassExW(&wc) == 0 {
                    let err = Win32Error::last();
                    match err.code() {
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
