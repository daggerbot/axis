/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::cell::Cell;

/// Window system event type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Event<W: 'static + Clone> {
    CloseRequest { window_id: W },
    Destroy { window_id: W },
    Update { update_mode: UpdateMode },
    VisibilityChange { window_id: W, visible: bool },
}

impl<W: 'static + Clone> Event<W> {
    /// Gets the window ID if applicable.
    pub fn window_id(&self) -> Option<&W> {
        match *self {
            Event::CloseRequest { ref window_id } => Some(window_id),
            Event::Destroy { ref window_id } => Some(window_id),
            Event::VisibilityChange { ref window_id, .. } => Some(window_id),
            _ => None,
        }
    }
}

/// Main loop state type.
pub struct MainLoop {
    quit: Cell<bool>,
    update_mode: Cell<UpdateMode>,
}

impl MainLoop {
    /// Returns true if the main loop is slated to break.
    pub fn is_quit_requested(&self) -> bool {
        self.quit.get()
    }

    /// Constructs a new main loop state.
    pub fn new(update_mode: UpdateMode) -> MainLoop {
        MainLoop {
            quit: Cell::new(false),
            update_mode: Cell::new(update_mode),
        }
    }

    /// Causes the main loop to break.
    pub fn quit(&self) {
        self.quit.set(true);
    }

    /// Changes the update mode.
    pub fn set_update_mode(&self, update_mode: UpdateMode) {
        self.update_mode.set(update_mode);
    }

    /// Returns the update mode.
    pub fn update_mode(&self) -> UpdateMode { self.update_mode.get() }
}

/// Determines when update events are triggered.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UpdateMode {
    /// A single update is triggered when the event queue is empty.
    Passive,
    /// Updates are continuously triggered when the event queue is empty.
    Active,
    /// If supported, an update is triggered once per v-blank. If not supported, this behaves the
    /// same as `Active`.
    Sync,
}
