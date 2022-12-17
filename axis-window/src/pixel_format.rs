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

/// Wrapper interface which allows an `IPixelFormat` object to be boxed.
pub trait IAnyPixelFormat {}

impl<T: IPixelFormat> IAnyPixelFormat for T {}

/// Window pixel format type. This is a boxed wrapper around an [`IPixelFormat`] object.
pub struct PixelFormat(pub(crate) Rc<dyn 'static + IAnyPixelFormat>);

/// Iterator over available pixel formats.
pub struct PixelFormats(pub(crate) Box<dyn 'static + Iterator<Item = PixelFormat>>);

impl Iterator for PixelFormats {
    type Item = PixelFormat;

    fn next(&mut self) -> Option<PixelFormat> {
        self.0.next()
    }
}
