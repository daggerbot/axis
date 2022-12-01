/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/// Determines when update events are generated.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UpdateKind {
    /// One update event is generated when the event queue is empty. This causes the main loop to
    /// block when no more events are available.
    Passive,

    /// Update events are constantly generated when the event queue is empty. The causes the main
    /// loop to never block.
    Active,

    /// Update events are generated at a time specifed by the driver. This ideally occurs during
    /// the monitor's v-blank, but this behavior may not be supported by all drivers. When
    /// unavailable, this behaves the same as [`Active`].
    VBlank,
}

/// Window system event type.
#[derive(Clone, Debug)]
pub enum Event<W: 'static + Clone> {
    Close { window_id: W },
    Destroy { window_id: W },
    Update { kind: UpdateKind },
    Visibility { window_id: W, visible: bool },
}

impl<W: 'static + Clone> Event<W> {
    /// Returns the event's window ID.
    pub fn window_id(&self) -> Option<&W> {
        match *self {
            Event::Close { ref window_id } => Some(window_id),
            Event::Destroy { ref window_id } => Some(window_id),
            Event::Visibility { ref window_id, .. } => Some(window_id),
            _ => None,
        }
    }
}
