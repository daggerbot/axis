/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use math::{Rect, Vector2};

use crate::image::{Image, ImageExt, ImageMut, OutOfBounds};

/// View of a portion of another `Image`.
pub struct Subimage<'a, I: 'a + ?Sized + Image> {
    pub(crate) parent: &'a I,
    pub(crate) region: Rect<usize>,
}

impl<'a, I: 'a + Image> Image for Subimage<'a, I> {
    type Pixel<'b> = I::Pixel<'b> where Self: 'b;

    unsafe fn pixel_unchecked<'b>(&'b self, pos: Vector2<usize>) -> Self::Pixel<'b> {
        self.parent.pixel_unchecked(pos + self.region.0)
    }

    fn size(&self) -> Vector2<usize> { self.region.size() }

    fn try_pixel<'b>(&'b self, pos: Vector2<usize>) -> Result<Self::Pixel<'b>, OutOfBounds> {
        Ok(self.parent.pixel(self.check_pixel_pos(pos)? + self.region.0))
    }
}

/// Mutable view of a portion of another `ImageMut`.
pub struct SubimageMut<'a, I: 'a + ?Sized + Image> {
    pub(crate) parent: &'a mut I,
    pub(crate) region: Rect<usize>,
}

impl<'a, I: 'a + Image> Image for SubimageMut<'a, I> {
    type Pixel<'b> = I::Pixel<'b> where Self: 'b;

    unsafe fn pixel_unchecked<'b>(&'b self, pos: Vector2<usize>) -> Self::Pixel<'b> {
        self.parent.pixel_unchecked(pos + self.region.0)
    }

    fn size(&self) -> Vector2<usize> { self.region.size() }

    fn try_pixel<'b>(&'b self, pos: Vector2<usize>) -> Result<Self::Pixel<'b>, OutOfBounds> {
        Ok(self.parent.pixel(self.check_pixel_pos(pos)? + self.region.0))
    }
}

impl<'a, I: 'a + ImageMut> ImageMut for SubimageMut<'a, I> {
    type PixelMut<'b> = I::PixelMut<'b> where Self: 'b;
    type PixelValue = I::PixelValue;

    unsafe fn pixel_mut_unchecked<'b>(&'b mut self, pos: Vector2<usize>) -> Self::PixelMut<'b> {
        self.parent.pixel_mut_unchecked(pos + self.region.0)
    }

    fn try_pixel_mut<'b>(&'b mut self, pos: Vector2<usize>)
        -> Result<Self::PixelMut<'b>, OutOfBounds>
    {
        Ok(self.parent.pixel_mut(self.check_pixel_pos(pos)? + self.region.0))
    }
}
