/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use math::Vector2;

use crate::image::{Image, ImageExt, OutOfBounds};

/// Image object which invokes a callback for each pixel requested.
pub struct Generate<T, F: Fn(Vector2<usize>) -> T> {
    callback: F,
    size: Vector2<usize>,
}

impl<T, F: Fn(Vector2<usize>) -> T> Image for Generate<T, F> {
    type Pixel<'a> = T where Self: 'a;

    fn height(&self) -> usize {
        self.size.y
    }

    unsafe fn get_pixel_unchecked<'a>(&'a self, pos: Vector2<usize>) -> T {
        (self.callback)(pos)
    }

    fn size(&self) -> Vector2<usize> {
        self.size
    }

    fn try_get_pixel<'a>(&'a self, pos: Vector2<usize>) -> Result<T, OutOfBounds> {
        Ok((self.callback)(self.check_pixel_pos(pos)?))
    }

    fn width(&self) -> usize {
        self.size.x
    }
}

/// Returns an image object which returns the default value for each pixel.
pub fn blank<T: Default>(size: Vector2<usize>) -> Generate<T, impl Fn(Vector2<usize>) -> T> {
    generate(size, |_| T::default())
}

/// Returns an image object which returns the same color for each pixel.
pub fn solid<'a, T: 'a + Clone>(
    size: Vector2<usize>, pixel: &'a T,
) -> Generate<T, impl 'a + Fn(Vector2<usize>) -> T> {
    generate(size, |_| pixel.clone())
}

/// Returns an image object which invokes a callback for each pixel requested.
pub fn generate<T, F>(size: Vector2<usize>, callback: F) -> Generate<T, F>
where
    F: Fn(Vector2<usize>) -> T,
{
    Generate { callback, size }
}
