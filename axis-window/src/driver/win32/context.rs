/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::c_void;
use std::iter::Once;
use std::marker::PhantomData;
use std::rc::Rc;

use crate::context::{IContext, MainLoop};
use crate::driver::win32::device::{Device, Devices};
use crate::driver::win32::pixel_format::PixelFormat;
use crate::driver::win32::util::Win32Error;
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

    /// Returns an [`Rc`] that is unique to this context.
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

    fn run<F: FnMut(Event<W>)>(&self, main_loop: &MainLoop, mut f: F) -> Result<()> {
        main_loop.clear_quit_flag();
        let _event_handler = EventHandler::push(self.event_manager.as_ref(), &f);

        'main_loop: while !main_loop.is_quit_requested() {
            // Handle our own queued events before polling the Win32 message queue.
            if let Some(event) = self.event_manager.pop() {
                (f)(event);
                continue 'main_loop;
            }

            unsafe {
                let mut msg = std::mem::zeroed();

                // Check for immediately available messages.
                match winapi::um::winuser::PeekMessageW(
                    &mut msg,
                    std::ptr::null_mut(),
                    0,
                    0,
                    winapi::um::winuser::PM_REMOVE,
                ) {
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
                        (f)(Event::Update {
                            kind: UpdateKind::Passive,
                        });
                        if main_loop.is_quit_requested() {
                            break 'main_loop;
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
                        (f)(Event::Update {
                            kind: UpdateKind::Active,
                        });
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

/// Thunk type used by [`EventHandler`].
#[derive(Clone, Copy)]
struct EventThunk<W: 'static + Clone> {
    f: unsafe fn(event: Event<W>, user_data: *mut c_void),
    user_data: *mut c_void,
}

impl<W: 'static + Clone> EventThunk<W> {
    unsafe fn invoke(&mut self, event: Event<W>) {
        (self.f)(event, self.user_data);
    }

    fn new<F: FnMut(Event<W>)>(f: &F) -> EventThunk<W> {
        EventThunk {
            f: EventThunk::thunk::<F>,
            user_data: f as *const F as *const c_void as *mut c_void,
        }
    }

    unsafe fn thunk<F: FnMut(Event<W>)>(event: Event<W>, user_data: *mut c_void) {
        let user_data = user_data as *mut F;
        (*user_data)(event);
    }
}

/// Handles incoming Win32 messages by either dispatching them to a Rust callback or queuing them.
pub struct EventManager<W: 'static + Clone> {
    callbacks: RefCell<Vec<Rc<RefCell<EventThunk<W>>>>>,
    queue: RefCell<VecDeque<Event<W>>>,
}

impl<W: 'static + Clone> EventManager<W> {
    pub fn new() -> EventManager<W> {
        EventManager {
            callbacks: RefCell::new(Vec::new()),
            queue: RefCell::new(VecDeque::new()),
        }
    }

    /// Pops the next queued event.
    pub fn pop(&self) -> Option<Event<W>> {
        self.queue.borrow_mut().pop_front()
    }

    /// Dispatches or enqueues the event.
    pub unsafe fn push(&self, event: Event<W>) {
        match self.top_callback() {
            None => self.queue.borrow_mut().push_back(event),
            Some(callback) => match callback.try_borrow_mut() {
                Ok(mut callback) => callback.invoke(event),
                Err(_) => self.queue.borrow_mut().push_back(event),
            },
        }
    }
}

impl<W: 'static + Clone> EventManager<W> {
    fn top_callback(&self) -> Option<Rc<RefCell<EventThunk<W>>>> {
        self.callbacks.borrow_mut().last().cloned()
    }
}

/// Holds an item on an [`EventManager`]'s callback stack.
struct EventHandler<'a, W: 'static + Clone> {
    event_manager: &'a EventManager<W>,
}

impl<'a, W: 'static + Clone> EventHandler<'a, W> {
    fn push<F: FnMut(Event<W>)>(event_manager: &'a EventManager<W>, f: &F) -> EventHandler<'a, W> {
        event_manager
            .callbacks
            .borrow_mut()
            .push(Rc::new(RefCell::new(EventThunk::new(f))));
        EventHandler { event_manager }
    }
}

impl<'a, W: 'static + Clone> Drop for EventHandler<'a, W> {
    fn drop(&mut self) {
        self.event_manager.callbacks.borrow_mut().pop();
    }
}
