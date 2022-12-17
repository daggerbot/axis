/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use math::{FromComposite, Vector2};

use crate::driver::x11::context::Context;
use crate::error::Result;
use crate::event::Event;
use crate::ffi::CBox;

impl<W: 'static + Clone> Context<W> {
    pub(crate) fn handle_event<F: Fn(Event<W>)>(
        &self, xevent: CBox<xcb_sys::xcb_generic_event_t>, f: &F,
    ) -> Result<()> {
        let xevent_ref: &xcb_sys::xcb_generic_event_t = xevent.as_ref();
        let xevent_ptr = xevent_ref as *const xcb_sys::xcb_generic_event_t;

        match (xevent.response_type & !0x80) as u32 {
            xcb_sys::XCB_CLIENT_MESSAGE => {
                self.handle_client_message(
                    unsafe { &(*(xevent_ptr as *const xcb_sys::xcb_client_message_event_t)) }, f)?;
            },

            xcb_sys::XCB_CONFIGURE_NOTIFY => {
                let ev = unsafe { *(xevent_ptr as *const xcb_sys::xcb_configure_notify_event_t) };
                if let Some(window) = self.window_manager().borrow().get(ev.window) {
                    let size = Vector2::new(ev.width, ev.height);

                    // Only emit a resize event if the size actually changed.
                    if window.update_size(size) {
                        f(Event::Resize {
                            window_id: window.id().clone(),
                            size: Vector2::from_composite(size),
                        });
                    }

                    // We'll usually get a pair of configure notify events if a window manager is
                    // active: one from the X server and one from the window manager. Each will give
                    // us a different value for the window's current position. Let's filter the
                    // event and pick the one that we actually want.
                    let is_server_event = xevent.response_type & 0x80 == 0;
                    if is_server_event == window.is_parent_root() {
                        let pos = Vector2::new(ev.x, ev.y);
                        if window.update_pos(pos) {
                            f(Event::Move {
                                window_id: window.id().clone(),
                                pos: Vector2::from_composite(pos),
                            });
                        }
                    }
                }
            },

            xcb_sys::XCB_DESTROY_NOTIFY => {
                let ev = unsafe { *(xevent_ptr as *const xcb_sys::xcb_destroy_notify_event_t) };
                if let Some(window) = self.window_manager().borrow_mut().expire(ev.window) {
                    f(Event::Destroy { window_id: window.id().clone() });
                }
            },

            xcb_sys::XCB_MAP_NOTIFY => {
                let ev = unsafe { *(xevent_ptr as *const xcb_sys::xcb_map_notify_event_t) };
                if let Some(window) = self.window_manager().borrow().get(ev.window).cloned() {
                    if window.update_visibility(true) {
                        f(Event::Visibility {
                            window_id: window.id().clone(),
                            visible: true,
                        });
                    }
                }
            },

            xcb_sys::XCB_REPARENT_NOTIFY => {
                let ev = unsafe { *(xevent_ptr as *const xcb_sys::xcb_reparent_notify_event_t) };
                if let Some(window) = self.window_manager().borrow().get(ev.window).cloned() {
                    // We'll use this to determine if a window manager has taken control of the
                    // window.
                    window.update_parent_xid(ev.parent);
                }
            },

            xcb_sys::XCB_UNMAP_NOTIFY => {
                let ev = unsafe { *(xevent_ptr as *const xcb_sys::xcb_unmap_notify_event_t) };
                if let Some(window) = self.window_manager().borrow().get(ev.window).cloned() {
                    if window.update_visibility(false) {
                        f(Event::Visibility {
                            window_id: window.id().clone(),
                            visible: false,
                        });
                    }
                }
            },

            _ => (),
        }

        Ok(())
    }
}

impl<W: 'static + Clone> Context<W> {
    fn handle_client_message<F: Fn(Event<W>)>(
        &self, event: &xcb_sys::xcb_client_message_event_t, f: &F,
    ) -> Result<()> {
        if event.type_ == self.atoms().WM_PROTOCOLS && event.format == 32 {
            let protocol = unsafe { event.data.data32[0] };

            // The name `WM_DELETE_WINDOW` is misleading. This is what we call a close request
            // event.
            if protocol == self.atoms().WM_DELETE_WINDOW {
                if let Some(window) = self.window_manager().borrow().get(event.window).cloned() {
                    f(Event::Close { window_id: window.id().clone() });
                }
            }
        }

        Ok(())
    }
}
