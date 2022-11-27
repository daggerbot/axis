/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::rc::Rc;

/// Interface for window system pixel formats.
pub trait IPixelFormat {}

/// Object interface for window system pixel formats.
pub trait IAnyPixelFormat {}

impl<T: IPixelFormat> IAnyPixelFormat for T {}

/// Opaque pixel format type.
pub struct PixelFormat(pub(crate) Rc<dyn 'static + IAnyPixelFormat>);

/// Opaque pixel format iterator.
pub struct PixelFormats(pub(crate) Box<dyn 'static + Iterator<Item = PixelFormat>>);

impl Iterator for PixelFormats {
    type Item = PixelFormat;

    fn next(&mut self) -> Option<PixelFormat> {
        self.0.next()
    }
}
