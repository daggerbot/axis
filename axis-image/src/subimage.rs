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

    unsafe fn get_pixel_unchecked<'b>(&'b self, pos: Vector2<usize>) -> Self::Pixel<'b> {
        self.parent.get_pixel_unchecked(pos + self.region.0)
    }

    fn height(&self) -> usize {
        self.region.height()
    }

    fn size(&self) -> Vector2<usize> {
        self.region.size()
    }

    fn try_get_pixel<'b>(&'b self, pos: Vector2<usize>) -> Result<Self::Pixel<'b>, OutOfBounds> {
        Ok(self
            .parent
            .get_pixel(self.check_pixel_pos(pos)? + self.region.0))
    }

    fn width(&self) -> usize {
        self.region.width()
    }
}

/// Mutable view of a portion of another `ImageMut`.
pub struct SubimageMut<'a, I: 'a + ?Sized + Image> {
    pub(crate) parent: &'a mut I,
    pub(crate) region: Rect<usize>,
}

impl<'a, I: 'a + Image> Image for SubimageMut<'a, I> {
    type Pixel<'b> = I::Pixel<'b> where Self: 'b;

    unsafe fn get_pixel_unchecked<'b>(&'b self, pos: Vector2<usize>) -> Self::Pixel<'b> {
        self.parent.get_pixel_unchecked(pos + self.region.0)
    }

    fn height(&self) -> usize {
        self.region.height()
    }

    fn size(&self) -> Vector2<usize> {
        self.region.size()
    }

    fn try_get_pixel<'b>(&'b self, pos: Vector2<usize>) -> Result<Self::Pixel<'b>, OutOfBounds> {
        Ok(self
            .parent
            .get_pixel(self.check_pixel_pos(pos)? + self.region.0))
    }

    fn width(&self) -> usize {
        self.region.width()
    }
}

impl<'a, I: 'a + ImageMut> ImageMut for SubimageMut<'a, I> {
    type PixelValue = I::PixelValue;

    unsafe fn set_pixel_unchecked(&mut self, pos: Vector2<usize>, pixel: I::PixelValue) {
        self.parent.set_pixel_unchecked(pos + self.region.0, pixel);
    }

    fn try_set_pixel(
        &mut self, pos: Vector2<usize>, pixel: I::PixelValue,
    ) -> Result<(), OutOfBounds> {
        self.parent
            .set_pixel(self.check_pixel_pos(pos)? + self.region.0, pixel);
        Ok(())
    }
}
