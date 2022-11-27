/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/// Window system event type.
#[derive(Clone, Debug)]
pub enum Event<W: 'static + Clone> {
    Close { window_id: W },
    Destroy { window_id: W },
    Visibility { window_id: W, visible: bool },
}

impl<W: 'static + Clone> Event<W> {
    /// Returns the event's window ID.
    pub fn window_id(&self) -> Option<&W> {
        match *self {
            Event::Close { ref window_id } => Some(window_id),
            Event::Destroy { ref window_id } => Some(window_id),
            Event::Visibility { ref window_id, .. } => Some(window_id),
        }
    }
}
