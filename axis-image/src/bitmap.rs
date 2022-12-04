/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::ops::Range;

use math::{DivCeil, TryMul, Vector2};

use crate::image::{Image, ImageExt, ImageMut, OutOfBounds};

/// Image type in which each pixel is a single bit.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Bitmap {
    buf: Vec<u8>,
    row_size: usize,
    size: Vector2<usize>,
}

impl Bitmap {
    /// Returns a reference to the underlying buffer.
    pub fn buf(&self) -> &[u8] {
        self.buf.as_ref()
    }

    /// Returns a mutable reference to the underlying buffer.
    pub fn buf_mut(&mut self) -> &mut [u8] {
        self.buf.as_mut()
    }

    /// Deconstructs the image and returns its underlying buffer.
    pub fn into_buf(self) -> Vec<u8> {
        self.buf
    }

    /// Returns a new bitmap with all pixels set to zero.
    pub fn new(size: Vector2<usize>) -> Bitmap {
        crate::generate::blank(size).to_bitmap()
    }

    /// Gets a reference to the specified row.
    pub fn row(&self, y: usize) -> &[u8] {
        self.try_row(y).unwrap()
    }

    /// Gets a mutable reference to the specified row.
    pub fn row_mut(&mut self, y: usize) -> &mut [u8] {
        self.try_row_mut(y).unwrap()
    }

    /// Gets a mutable reference to the specified row without bounds checking.
    pub unsafe fn row_mut_unchecked(&mut self, y: usize) -> &mut [u8] {
        let index = self.row_index_unchecked(y);
        self.buf.get_unchecked_mut(index)
    }

    /// Gets a reference to the specified row without bounds checking.
    pub unsafe fn row_unchecked(&self, y: usize) -> &[u8] {
        self.buf.get_unchecked(self.row_index_unchecked(y))
    }

    /// Gets a reference to the specified row.
    pub fn try_row(&self, y: usize) -> Result<&[u8], OutOfBounds> {
        Ok(&self.buf[self.try_row_index(y)?])
    }

    /// Gets a mutable reference to the specified row.
    pub fn try_row_mut(&mut self, y: usize) -> Result<&mut [u8], OutOfBounds> {
        let index = self.try_row_index(y)?;
        Ok(&mut self.buf[index])
    }
}

impl Bitmap {
    /// Returns the byte index and the bit shift index of the specified pixel.
    fn pixel_index_unchecked(&self, pos: Vector2<usize>) -> (usize, u8) {
        let index = pos.y * self.row_size + pos.x / 8;
        let shift = 7 - (pos.x % 8) as u8;
        (index, shift)
    }

    /// Returns the byte index of the specified row.
    fn row_index_unchecked(&self, y: usize) -> Range<usize> {
        let start = y * self.row_size;
        start..(start + self.row_size)
    }

    /// Returns the byte index and the bit shift index of the specified pixel.
    fn try_pixel_index(&self, pos: Vector2<usize>) -> Result<(usize, u8), OutOfBounds> {
        self.check_pixel_pos(pos)?;
        Ok(self.pixel_index_unchecked(pos))
    }

    /// Returns the byte index of the specified row.
    fn try_row_index(&self, y: usize) -> Result<Range<usize>, OutOfBounds> {
        if y >= self.size.y {
            return Err(OutOfBounds);
        }
        Ok(self.row_index_unchecked(y))
    }
}

impl<'a, T: 'a + Into<bool>, I: 'a + Image<Pixel<'a> = T>> From<&'a I> for Bitmap {
    /// Renders the source image as a `[Bitmap]`.
    fn from(src: &'a I) -> Bitmap {
        let size = src.size();
        let mut bitmap = Bitmap {
            buf: Vec::new(),
            row_size: DivCeil::div_ceil(size.x, 8),
            size,
        };

        let buf_size = bitmap.row_size.try_mul(size.y).unwrap();
        bitmap.buf.reserve_exact(buf_size);

        for y in 0..size.y {
            let mut byte = 0;
            let mut shift = 7;

            for x in 0..size.x {
                if src.get_pixel(Vector2::new(x, y)).into() {
                    byte |= 1 << shift;
                }
                if shift == 0 {
                    bitmap.buf.push(byte);
                    byte = 0;
                    shift = 7;
                }
                shift -= 1;
            }

            if shift != 7 {
                bitmap.buf.push(byte);
            }
        }

        bitmap
    }
}

impl Image for Bitmap {
    type Pixel<'a> = bool where Self: 'a;

    unsafe fn get_pixel_unchecked<'a>(&'a self, pos: Vector2<usize>) -> bool {
        let (index, shift) = self.pixel_index_unchecked(pos);
        (self.buf[index] & (1 << shift)) != 0
    }

    fn height(&self) -> usize {
        self.size.y
    }

    fn size(&self) -> Vector2<usize> {
        self.size
    }

    fn try_get_pixel<'a>(&'a self, pos: Vector2<usize>) -> Result<bool, OutOfBounds> {
        let (index, shift) = self.try_pixel_index(pos)?;
        Ok((self.buf[index] & (1 << shift)) != 0)
    }

    fn width(&self) -> usize {
        self.size.x
    }
}

impl ImageMut for Bitmap {
    type PixelValue = bool;

    unsafe fn set_pixel_unchecked(&mut self, pos: Vector2<usize>, pixel: bool) {
        let (index, shift) = self.pixel_index_unchecked(pos);
        let byte = 1 << shift;
        if pixel {
            *self.buf.get_unchecked_mut(index) |= byte;
        } else {
            *self.buf.get_unchecked_mut(index) &= !byte;
        }
    }

    fn try_set_pixel(&mut self, pos: Vector2<usize>, pixel: bool) -> Result<(), OutOfBounds> {
        let (index, shift) = self.try_pixel_index(pos)?;
        let byte = 1 << shift;
        if pixel {
            self.buf[index] |= byte;
        } else {
            self.buf[index] &= !byte;
        }
        Ok(())
    }
}
