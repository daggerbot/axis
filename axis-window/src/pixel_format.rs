/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::any::Any;
use std::rc::Rc;

/// Pixel format interface.
pub trait IPixelFormat: Clone + Eq {}

/// Internal interface for [PixelFormat].
trait IPixelFormatObject: 'static {
    fn eq(&self, rhs: &dyn Any) -> bool;
}

impl<T: 'static + IPixelFormat> IPixelFormatObject for T {
    fn eq(&self, rhs: &dyn Any) -> bool {
        match rhs.downcast_ref::<T>() {
            None => false,
            Some(rhs) => *self == *rhs,
        }
    }
}

/// Boxed pixel format type.
#[derive(Clone)]
pub struct PixelFormat {
    inner: Rc<dyn IPixelFormatObject>,
}

impl PixelFormat {
    pub(crate) fn new<T: 'static + IPixelFormat>(inner: T) -> PixelFormat {
        PixelFormat { inner: Rc::new(inner) }
    }
}

impl Eq for PixelFormat {}

impl IPixelFormat for PixelFormat {}

impl PartialEq for PixelFormat {
    fn eq(&self, rhs: &PixelFormat) -> bool {
        self.inner.eq(rhs)
    }
}
