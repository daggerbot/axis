/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::cell::Cell;
use std::iter::Once;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::context::{IContext, MainLoop};
use crate::driver::win32::device::{Device, Devices};
use crate::driver::win32::error::Win32Error;
use crate::driver::win32::event::{EventHandler, EventManager};
use crate::driver::win32::pixel_format::PixelFormat;
use crate::driver::win32::window::{Window, WindowBuilder};
use crate::error::Result;
use crate::event::{Event, UpdateKind};

/// Context to the Win32 window system.
pub struct Context<W: 'static + Clone> {
    event_manager: Rc<EventManager<W>>,
    _phantom: PhantomData<W>,
    unique: Rc<()>,
}

impl<W: 'static + Clone> Context<W> {
    /// Opens a window system context for the current thread.
    ///
    /// It should be noted that multiple contexts may be opened on the same thread, but polling
    /// events from one may discard events for windows created by another.
    pub fn open() -> Result<Context<W>> {
        Ok(Context {
            event_manager: Rc::new(EventManager::new()),
            _phantom: PhantomData,
            unique: Rc::new(()),
        })
    }
}

impl<W: 'static + Clone> Context<W> {
    pub(crate) fn event_manager(&self) -> &Rc<EventManager<W>> {
        &self.event_manager
    }

    /// Returns an [`Rc`] that is unique to this context. Used to compare whether two contexts or
    /// devices are the same. There has to be a better way to do this.
    pub(crate) fn unique(&self) -> &Rc<()> {
        &self.unique
    }
}

impl<W: 'static + Clone> IContext for Context<W> {
    type Device = Device<W>;
    type Devices = Devices<W>;
    type PixelFormat = PixelFormat;
    type PixelFormats = Once<PixelFormat>;
    type Window = Window<W>;
    type WindowBuilder = WindowBuilder<W>;
    type WindowId = W;

    fn default_device(&self) -> Device<W> {
        Device::new(self)
    }

    fn devices(&self) -> Devices<W> {
        std::iter::once(Device::new(self))
    }

    fn run<F: Fn(Event<W>)>(&self, main_loop: &MainLoop, callback: F) -> Result<()> {
        main_loop.clear_quit_flag();
        let mut msg = unsafe { std::mem::zeroed() };

        // Flag which indicates whether another Update event can be sent. This prevents signalling
        // two or more update events in a row unnecessarily if the main loop is passive.
        let update_ready = Cell::new(true);

        // Callback wrapper. Most of the time an event is sent, we are ready for another update
        // event.
        let f = |event| {
            (callback)(event);
            update_ready.set(true);
        };

        let _event_handler = unsafe { EventHandler::push(self.event_manager.as_ref(), &f) };

        'main_loop: while !main_loop.is_quit_requested() {
            // Handle our own queued events before polling the Win32 message queue.
            if let Some(event) = self.event_manager.pop() {
                (f)(event);
                continue 'main_loop;
            }

            unsafe {
                // Check for immediately available messages.
                match winapi::um::winuser::PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0,
                                                        winapi::um::winuser::PM_REMOVE)
                {
                    0 => (),
                    _ => {
                        if msg.message == winapi::um::winuser::WM_QUIT {
                            break 'main_loop;
                        }
                        winapi::um::winuser::TranslateMessage(&mut msg);
                        winapi::um::winuser::DispatchMessageW(&mut msg);
                        continue 'main_loop;
                    },
                }

                // Nothing immediately available. Wait or handle an update message.
                match main_loop.update_kind() {
                    UpdateKind::Passive => {
                        if update_ready.take() {
                            (callback)(Event::Update { kind: UpdateKind::Passive });
                            if main_loop.is_quit_requested() {
                                break 'main_loop;
                            }
                        }

                        match winapi::um::winuser::GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0)
                        {
                            -1 => return Err(err!(SystemError("GetMessageW"): Win32Error::last())),
                            0 => break 'main_loop,
                            _ => {
                                winapi::um::winuser::TranslateMessage(&mut msg);
                                winapi::um::winuser::DispatchMessageW(&mut msg);
                            },
                        }
                    },

                    UpdateKind::Active | UpdateKind::VBlank => {
                        (callback)(Event::Update { kind: UpdateKind::Active });
                    },
                }
            }
        }

        Ok(())
    }
}

impl<W: 'static + Clone> Eq for Context<W> {}

impl<W: 'static + Clone> PartialEq for Context<W> {
    fn eq(&self, rhs: &Context<W>) -> bool {
        &*self.unique as *const () == &*rhs.unique as *const ()
    }
}
