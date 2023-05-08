/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::mem::MaybeUninit;
use std::os::raw::c_void;
use std::rc::Rc;

use crate::client::IClient;
use crate::driver::win32::pixel_format::PixelFormat;
use crate::driver::win32::window::{Window, WindowBuilder, WindowClassManager};
use crate::error::Result;
use crate::event::{Event, MainLoop, UpdateMode};

/// Win32 window system client type.
pub struct Client<W: 'static + Clone> {
    event_manager: Rc<EventManager<W>>,
    window_class_name: Rc<Vec<u16>>,
}

impl<W: 'static + Clone> Client<W> {
    /// Opens a window system client for the current thread.
    pub fn open() -> Result<Client<W>> {
        Ok(Client {
            event_manager: Rc::new(EventManager::new()),
            window_class_name: Rc::new(WindowClassManager::get().lock()?.register::<W>()?),
        })
    }
}

impl<W: 'static + Clone> Client<W> {
    pub(crate) fn event_manager(&self) -> &Rc<EventManager<W>> { &self.event_manager }

    pub(crate) fn window_class_name(&self) -> &Rc<Vec<u16>> { &self.window_class_name }
}

impl<W: 'static + Clone> IClient for Client<W> {
    type PixelFormat = PixelFormat;
    type Window = Window<W>;
    type WindowBuilder = WindowBuilder<W>;
    type WindowId = W;

    fn default_pixel_format(&self) -> PixelFormat {
        PixelFormat::default()
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

        unsafe {
            let mut msg = MaybeUninit::zeroed().assume_init();
            let event_handler = EventHandler::push(self.event_manager.as_ref(), &f);

            // Handle events that were processed and queued.
            'queue_loop: while !main_loop.is_quit_requested() {
                match self.event_manager.pop() {
                    None => break 'queue_loop,
                    Some(event) => event_handler.dispatch(event),
                }
            }

            'main_loop: while !main_loop.is_quit_requested() {
                // Handle pending Win32 messages.
                while winapi::um::winuser::PeekMessageW(
                    &mut msg, std::ptr::null_mut(), 0, 0, winapi::um::winuser::PM_REMOVE) != 0
                {
                    if msg.message == winapi::um::winuser::WM_QUIT {
                        break 'main_loop;
                    }

                    winapi::um::winuser::TranslateMessage(&msg);
                    winapi::um::winuser::DispatchMessageW(&msg);

                    if main_loop.is_quit_requested() {
                        break 'main_loop;
                    }
                }

                // Handle update event and wait for more messages.
                match main_loop.update_mode() {
                    UpdateMode::Passive => {
                        if need_update.take() {
                            event_handler.dispatch(Event::Update {
                                update_mode: UpdateMode::Passive,
                            });
                        }

                        if main_loop.is_quit_requested() {
                            break 'main_loop;
                        }

                        match winapi::um::winuser::GetMessageW(&mut msg, std::ptr::null_mut(),
                                                               0, 0)
                        {
                            -1 => return Err(err!(RuntimeError("GetMessageW"): ??w)),
                            0 => break 'main_loop,
                            _ => {
                                winapi::um::winuser::TranslateMessage(&msg);
                                winapi::um::winuser::DispatchMessageW(&msg);
                            },
                        }
                    },

                    UpdateMode::Active | UpdateMode::Sync => {
                        event_handler.dispatch(Event::Update {
                            update_mode: UpdateMode::Active,
                        });
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

/// Handles window system events.
pub struct EventManager<W: 'static + Clone> {
    dispatch_stack: RefCell<Vec<EventDispatch<W>>>,
    event_queue: RefCell<VecDeque<Event<W>>>,
}

impl<W: 'static + Clone> EventManager<W> {
    /// Invokes the top event handler, or enqueues the event if no handler is present.
    pub fn push(&self, event: Event<W>) {
        let dispatch = self.dispatch_stack.borrow().last().cloned();
        match dispatch {
            None => self.event_queue.borrow_mut().push_back(event),
            Some(dispatch) => {
                unsafe {
                    dispatch.dispatch(event);
                }
            },
        }
    }
}

impl<W: 'static + Clone> EventManager<W> {
    fn new() -> EventManager<W> {
        EventManager {
            dispatch_stack: RefCell::new(Vec::new()),
            event_queue: RefCell::new(VecDeque::new()),
        }
    }

    fn pop(&self) -> Option<Event<W>> {
        self.event_queue.borrow_mut().pop_front()
    }
}

/// Unsafe event handler wrapper.
#[derive(Clone)]
struct EventDispatch<W: 'static + Clone> {
    thunk: unsafe fn(user_data: *const c_void, event: Event<W>),
    user_data: *const c_void,
}

impl<W: 'static + Clone> EventDispatch<W> {
    unsafe fn dispatch(&self, event: Event<W>) {
        (self.thunk)(self.user_data, event);
    }

    unsafe fn new<F: Fn(Event<W>)>(f: &F) -> EventDispatch<W> {
        EventDispatch {
            thunk: EventDispatch::<W>::thunk::<F>,
            user_data: f as *const F as *const _,
        }
    }

    unsafe fn thunk<F: Fn(Event<W>)>(user_data: *const c_void, event: Event<W>) {
        (*(user_data as *const F))(event);
    }
}

/// Holds an item on the event dispatch queue.
struct EventHandler<'a, W: 'static + Clone> {
    dispatch: EventDispatch<W>,
    manager: &'a EventManager<W>,
    top: usize,
}

impl<'a, W: 'static + Clone> EventHandler<'a, W> {
    unsafe fn dispatch(&self, event: Event<W>) {
        self.dispatch.dispatch(event);
    }

    unsafe fn push<F: Fn(Event<W>)>(manager: &'a EventManager<W>, f: &F) -> EventHandler<'a, W> {
        let mut dispatch_stack = manager.dispatch_stack.borrow_mut();
        let dispatch = EventDispatch::new(f);
        let top = dispatch_stack.len();
        dispatch_stack.push(dispatch.clone());
        EventHandler { dispatch, manager, top }
    }
}

impl<'a, W: 'static + Clone> Drop for EventHandler<'a, W> {
    fn drop(&mut self) {
        let mut dispatch_stack = self.manager.dispatch_stack.borrow_mut();
        while dispatch_stack.len() > self.top {
            dispatch_stack.pop();
        }
    }
}
