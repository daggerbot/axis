/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::ops::Range;

use math::{TryMul, Vector2};

use crate::image::{Image, ImageExt, ImageMut, OutOfBounds};

/// Owned image type backed by a `Vec` in which pixels are stored sequentially.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct VecImage<T> {
    buf: Vec<T>,
    size: Vector2<usize>,
}

impl<T> VecImage<T> {
    /// Returns a reference to the underlying buffer.
    pub fn buf(&self) -> &[T] {
        &self.buf
    }

    /// Returns a mutable reference to the underlying buffer.
    pub fn buf_mut(&mut self) -> &mut [T] {
        &mut self.buf
    }

    /// Deconstructs the image and returns its buffer.
    pub fn into_buf(self) -> Vec<T> {
        self.buf
    }

    /// Returns a new image with all pixels initializes to the default value for `T`.
    pub fn new(size: Vector2<usize>) -> VecImage<T>
    where
        T: Default,
    {
        crate::generate::blank(size).to_vec_image()
    }

    /// Returns a new image with all pixels initialized to the specified value.
    pub fn new_solid(size: Vector2<usize>, pixel: &T) -> VecImage<T>
    where
        T: Clone,
    {
        crate::generate::solid(size, pixel).to_vec_image()
    }

    /// Gets a reference to the specified row.
    pub fn row(&self, y: usize) -> &[T] {
        self.try_row(y).unwrap()
    }

    /// Gets a mutable reference to the specifed row.
    pub fn row_mut(&mut self, y: usize) -> &mut [T] {
        self.try_row_mut(y).unwrap()
    }

    /// Gets a mutable reference to the specified row without bounds checking.
    pub unsafe fn row_mut_unchecked(&mut self, y: usize) -> &mut [T] {
        let index = self.row_index_unchecked(y);
        self.buf.get_unchecked_mut(index)
    }

    /// Gets a reference to the specified row without bounds checking.
    pub unsafe fn row_unchecked(&self, y: usize) -> &[T] {
        self.buf.get_unchecked(self.row_index_unchecked(y))
    }

    /// Attempts to get a reference to the specified row.
    pub fn try_row(&self, y: usize) -> Result<&[T], OutOfBounds> {
        Ok(&self.buf[self.try_row_index(y)?])
    }

    /// Attempts to get a mutable reference to the specified row.
    pub fn try_row_mut(&mut self, y: usize) -> Result<&mut [T], OutOfBounds> {
        let index = self.try_row_index(y)?;
        Ok(&mut self.buf[index])
    }
}

impl<T> VecImage<T> {
    fn pixel_index_unchecked(&self, pos: Vector2<usize>) -> usize {
        pos.y * self.size.x + pos.x
    }

    fn row_index_unchecked(&self, y: usize) -> Range<usize> {
        let start = y * self.size.x;
        start..(start + self.size.x)
    }

    fn try_pixel_index(&self, pos: Vector2<usize>) -> Result<usize, OutOfBounds> {
        Ok(self.pixel_index_unchecked(self.check_pixel_pos(pos)?))
    }

    fn try_row_index(&self, y: usize) -> Result<Range<usize>, OutOfBounds> {
        if y >= self.size.y {
            return Err(OutOfBounds);
        }
        Ok(self.row_index_unchecked(y))
    }
}

impl<'a, T, U: 'a + Into<T>, I: ?Sized + Image<Pixel<'a> = U>> From<&'a I> for VecImage<T> {
    /// Renders the source image as a `[VecImage]`.
    fn from(source: &'a I) -> VecImage<T> {
        let mut image = VecImage {
            buf: Vec::new(),
            size: source.size(),
        };

        let buf_size = image.size.x.try_mul(image.size.y).unwrap();
        image.buf.reserve_exact(buf_size);

        for y in 0..image.size.y {
            for x in 0..image.size.x {
                image.buf.push(source.get_pixel(Vector2::new(x, y)).into());
            }
        }

        image
    }
}

impl<T> Image for VecImage<T> {
    type Pixel<'a> = &'a T where Self: 'a;

    unsafe fn get_pixel_unchecked<'a>(&'a self, pos: Vector2<usize>) -> &'a T {
        self.buf.get_unchecked(self.pixel_index_unchecked(pos))
    }

    fn height(&self) -> usize {
        self.size.y
    }

    fn size(&self) -> Vector2<usize> {
        self.size
    }

    fn try_get_pixel<'a>(&'a self, pos: Vector2<usize>) -> Result<&'a T, OutOfBounds> {
        Ok(&self.buf[self.try_pixel_index(pos)?])
    }

    fn width(&self) -> usize {
        self.size.x
    }
}

impl<T> ImageMut for VecImage<T> {
    type PixelValue = T;

    unsafe fn set_pixel_unchecked(&mut self, pos: Vector2<usize>, pixel: T) {
        let index = self.pixel_index_unchecked(pos);
        *self.buf.get_unchecked_mut(index) = pixel;
    }

    fn try_set_pixel(&mut self, pos: Vector2<usize>, pixel: T) -> Result<(), OutOfBounds> {
        let index = self.try_pixel_index(pos)?;
        self.buf[index] = pixel;
        Ok(())
    }
}
