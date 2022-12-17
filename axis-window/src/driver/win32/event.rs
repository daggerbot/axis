/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/*
 * NOTE: Some of the below code is witchcraft. We push an event handler on a stack that each window
 * has access to from its WndProc. This may sound fine at first. The problem is that if for some
 * reason our EventHandler is leaked (leaks are not considered unsafe in Rust), then the window will
 * invoke a callback that no longer exists. I can't think of any scenario where this would actually
 * happen, but I still don't like it.
 */

use std::cell::RefCell;
use std::collections::VecDeque;
use std::ffi::c_void;
use std::marker::PhantomData;
use std::rc::Rc;

use math::Vector2;

use crate::driver::win32::window::WindowShared;
use crate::event::Event;
use crate::Coord;

/// Thunk type used by [`EventHandler`]. This seemed to be the best way to invoke the callback
/// opaquely, as boxed functions in Rust are still pretty wonky.
#[derive(Clone, Copy)]
struct EventThunk<W: 'static + Clone> {
    f: unsafe fn(event: Event<W>, user_data: *const c_void),
    user_data: *const c_void,
}

impl<W: 'static + Clone> EventThunk<W> {
    /// Invokes the thunk callback.
    unsafe fn invoke(&self, event: Event<W>) {
        (self.f)(event, self.user_data);
    }

    /// Constructs an event thunk object. Does nothing unsafe *yet*.
    fn new<F: Fn(Event<W>)>(f: &F) -> EventThunk<W> {
        EventThunk {
            f: EventThunk::thunk::<F>,
            user_data: f as *const F as *const c_void,
        }
    }

    /// Thunks into the callback in a very C-ish way. Feel free to change this if there's a better
    /// way.
    unsafe fn thunk<F: Fn(Event<W>)>(event: Event<W>, user_data: *const c_void) {
        (*(user_data as *const F))(event);
    }
}

/// Handles incoming Win32 messages by either dispatching them to a Rust callback or queuing them
/// for when the callback is ready for them.
pub struct EventManager<W: 'static + Clone> {
    callbacks: RefCell<Vec<Rc<EventThunk<W>>>>,
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

    /// Dispatches the event to the top callback if it is available, or otherwise queues the event
    /// for later processing.
    pub unsafe fn push(&self, event: Event<W>) {
        match self.top_callback() {
            None => self.queue.borrow_mut().push_back(event),
            Some(callback) => callback.invoke(event),
        }
    }
}

impl<W: 'static + Clone> EventManager<W> {
    /// Returns the callback thunk at the top of the callback stack.
    fn top_callback(&self) -> Option<Rc<EventThunk<W>>> {
        self.callbacks.borrow().last().cloned()
    }
}

/// Holds an item on an [`EventManager`]'s callback stack. The destructor pops this callback (and
/// any others that might be above it for whatever reason) from that stack. If an `EventHandler` is
/// leaked for some reason (not sure why it ever would), then a world of undefined behavior is
/// unleashed on the unsuspecting world.
pub struct EventHandler<'a, 'f, W: 'static + Clone> {
    event_manager: &'a EventManager<W>,
    _phantom: PhantomData<&'f ()>,
    top: usize,
}

impl<'a, 'f, W: 'static + Clone> EventHandler<'a, 'f, W> {
    /// Pushes a callback on top of the event manager's stack and returns an `EventHandler` that
    /// will pop it later.
    pub unsafe fn push<F: 'f + Fn(Event<W>)>(event_manager: &'a EventManager<W>, f: &'f F)
        -> EventHandler<'a, 'f, W>
    {
        let top = event_manager.callbacks.borrow().len();
        event_manager.callbacks.borrow_mut().push(Rc::new(EventThunk::new(f)));

        EventHandler {
            event_manager,
            _phantom: PhantomData,
            top,
        }
    }
}

impl<'a, 'f, W: 'static + Clone> Drop for EventHandler<'a, 'f, W> {
    fn drop(&mut self) {
        let mut callbacks = self.event_manager.callbacks.borrow_mut();
        while callbacks.len() > self.top {
            callbacks.pop();
        }
    }
}

/// Window message handler.
pub unsafe extern "system" fn wndproc<W: 'static + Clone>(hwnd: winapi::shared::windef::HWND,
                                                          msg: u32, wparam: usize, lparam: isize)
                                                          -> isize
{
    match msg {
        winapi::um::winuser::WM_CLOSE => {
            if let Some(window) = WindowShared::<W>::from_hwnd(hwnd) {
                window.event_manager().push(Event::Close { window_id: window.id().clone() });
            }
            0
        },

        winapi::um::winuser::WM_DESTROY => {
            if let Some(window) = WindowShared::<W>::expire(hwnd) {
                window.event_manager().push(Event::Destroy { window_id: window.id().clone() });
            }
            0
        },

        winapi::um::winuser::WM_MOVE => {
            if let Some(window) = WindowShared::<W>::from_hwnd(hwnd) {
                let x = winapi::shared::minwindef::LOWORD(lparam as u32) as i16;
                let y = winapi::shared::minwindef::HIWORD(lparam as u32) as i16;
                window.event_manager().push(Event::Move {
                    window_id: window.id().clone(),
                    pos: Vector2 {
                        x: Coord::from(x),
                        y: Coord::from(y),
                    },
                });
            }
            0
        },

        winapi::um::winuser::WM_SHOWWINDOW => {
            if let Some(window) = WindowShared::<W>::from_hwnd(hwnd) {
                window.event_manager().push(Event::Visibility {
                    visible: wparam != 0,
                    window_id: window.id().clone(),
                });
            }
            0
        },

        winapi::um::winuser::WM_SIZE => {
            if let Some(window) = WindowShared::<W>::from_hwnd(hwnd) {
                let width = winapi::shared::minwindef::LOWORD(lparam as u32);
                let height = winapi::shared::minwindef::HIWORD(lparam as u32);
                window.event_manager().push(Event::Resize {
                    window_id: window.id().clone(),
                    size: Vector2 {
                        x: Coord::from(width),
                        y: Coord::from(height),
                    },
                });
            }
            0
        },

        _ => winapi::um::winuser::DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
