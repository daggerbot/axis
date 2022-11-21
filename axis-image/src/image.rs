/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::{Deref, DerefMut};

use math::{Rect, Vector2};

use crate::map::{Cloned, Copied, Map};
use crate::subimage::{Subimage, SubimageMut};
use crate::vec_image::VecImage;

/// Trait for getting pixels from an image.
pub trait Image: Sized {
    /// The image's pixel type. Has a lifetime parameter because some image types may prefer to
    /// return a reference to a pixel.
    type Pixel<'a>: Sized where Self: 'a;

    /// Returns an image structure that clones each requested pixel.
    fn cloned<'a>(&'a self) -> Cloned<'a, <Self::Pixel<'a> as Deref>::Target, Self>
    where
        Self::Pixel<'a>: Deref,
        <Self::Pixel<'a> as Deref>::Target: Clone,
    {
        Cloned { parent: self }
    }

    /// Returns an image structure that copies each requested pixel.
    fn copied<'a>(&'a self) -> Copied<'a, <Self::Pixel<'a> as Deref>::Target, Self>
    where
        Self::Pixel<'a>: Deref,
        <Self::Pixel<'a> as Deref>::Target: Copy,
    {
        Copied { parent: self }
    }

    /// Returns a PNG encoder for the image.
    #[cfg(feature = "png")]
    fn encode_png<'i, 'p>(&'i self) -> crate::codec::png::Encoder<'i, 'p, Self>
    where
        Self::Pixel<'i>: crate::codec::png::Pixel,
    {
        crate::codec::png::Encoder::new(self)
    }

    /// Gets a copy of the referenced pixel at the specified position.
    fn get_pixel<'a>(&'a self, pos: Vector2<usize>) -> <Self::Pixel<'a> as Deref>::Target
    where
        Self::Pixel<'a>: Deref,
        <Self::Pixel<'a> as Deref>::Target: Copy,
    {
        *self.pixel(pos)
    }

    /// Returns the image's height in pixels.
    fn height(&self) -> usize { self.size().y }

    /// Uses a callback to map pixel values.
    fn map<'a, T: 'a, F: Fn(Self::Pixel<'a>) -> T>(&'a self, f: F) -> Map<'a, Self, T, F> {
        Map { callback: f, parent: self }
    }

    /// Gets the pixel at the specified position.
    fn pixel<'a>(&'a self, pos: Vector2<usize>) -> Self::Pixel<'a> {
        self.try_pixel(pos).unwrap()
    }

    /// Gets the pixel at the specified position without bounds checking.
    unsafe fn pixel_unchecked<'a>(&'a self, pos: Vector2<usize>) -> Self::Pixel<'a> {
        self.pixel(pos)
    }

    /// Gets a copy of the referenced pixel at the specified position without bounds checking.
    unsafe fn get_pixel_unchecked<'a>(&'a self, pos: Vector2<usize>)
        -> <Self::Pixel<'a> as Deref>::Target
    where
        Self::Pixel<'a>: Deref,
        <Self::Pixel<'a> as Deref>::Target: Copy,
    {
        *self.pixel_unchecked(pos)
    }

    /// Returns the image's size in pixels.
    fn size(&self) -> Vector2<usize>;

    /// Gets a view of a region within the image.
    fn subimage<'a>(&'a self, region: Rect<usize>) -> Subimage<'a, Self> {
        self.try_subimage(region).unwrap()
    }

    /// Gets a view of a region within the image without bounds checking.
    fn subimage_unchecked<'a>(&'a self, region: Rect<usize>) -> Subimage<'a, Self> {
        Subimage { parent: self, region }
    }

    /// Renders the image's contents to a `VecImage`.
    fn to_vec_image<'a>(&'a self) -> VecImage<Self::Pixel<'a>> {
        VecImage::from(self)
    }

    /// Attempts to get a copy of the referenced pixel at the specified position.
    fn try_get_pixel<'a>(&'a self, pos: Vector2<usize>)
        -> Result<<Self::Pixel<'a> as Deref>::Target, OutOfBounds>
    where
        Self::Pixel<'a>: Deref,
        <Self::Pixel<'a> as Deref>::Target: Copy,
    {
        Ok(*self.try_pixel(pos)?)
    }

    /// Attempts to get the pixel at the specified position.
    fn try_pixel<'a>(&'a self, pos: Vector2<usize>) -> Result<Self::Pixel<'a>, OutOfBounds>;

    /// Attempts to get a view of a region within the image.
    fn try_subimage<'a>(&'a self, region: Rect<usize>) -> Result<Subimage<'a, Self>, OutOfBounds> {
        Ok(Subimage {
            parent: self,
            region: self.check_pixel_region(region)?,
        })
    }

    /// Returns the image's width in pixels.
    fn width(&self) -> usize { self.size().x }
}

/// Extension functions for images.
pub trait ImageExt: Image {
    /// Returns `pos` if it is a valid pixel index, or `OutOfBounds` if not.
    fn check_pixel_pos(&self, pos: Vector2<usize>) -> Result<Vector2<usize>, OutOfBounds> {
        let size = self.size();
        if pos.x >= size.x || pos.y >= size.y {
            return Err(OutOfBounds);
        }
        Ok(pos)
    }

    /// Returns `region` if it is a valid region within the image, or `OutOfBounds` if not.
    fn check_pixel_region(&self, region: Rect<usize>) -> Result<Rect<usize>, OutOfBounds> {
        let size = self.size();
        if !region.is_ordered() || region.1.x > size.x || region.1.y > size.y {
            return Err(OutOfBounds);
        }
        Ok(region)
    }
}

impl<T: ?Sized + Image> ImageExt for T {}

/// Trait for changing the pixels in an image.
pub trait ImageMut: Image {
    /// Mutable reference to a pixel.
    type PixelMut<'a>: DerefMut<Target = Self::PixelValue> where Self: 'a;

    /// Pixel value type.
    type PixelValue;

    /// Gets a mutable reference to the pixel at the specified position.
    fn pixel_mut<'a>(&'a mut self, pos: Vector2<usize>) -> Self::PixelMut<'a> {
        self.try_pixel_mut(pos).unwrap()
    }

    /// Gets a mutable reference to the pixel at the specified position without bounds checking.
    unsafe fn pixel_mut_unchecked<'a>(&'a mut self, pos: Vector2<usize>) -> Self::PixelMut<'a> {
        self.pixel_mut(pos)
    }

    /// Sets the pixel at the specified position.
    fn set_pixel(&mut self, pos: Vector2<usize>, pixel: Self::PixelValue) {
        *self.pixel_mut(pos) = pixel;
    }

    /// Sets the pixel at the specified position without bounds checking.
    unsafe fn set_pixel_unchecked(&mut self, pos: Vector2<usize>, pixel: Self::PixelValue) {
        *self.pixel_mut_unchecked(pos) = pixel;
    }

    /// Gets a view of a region within the image.
    fn subimage_mut<'a>(&'a mut self, region: Rect<usize>) -> SubimageMut<'a, Self> {
        self.try_subimage_mut(region).unwrap()
    }

    /// Gets a view of a region within the image without bounds checking.
    fn subimage_mut_unchecked<'a>(&'a mut self, region: Rect<usize>) -> SubimageMut<'a, Self> {
        SubimageMut { parent: self, region }
    }

    /// Attempts to get a mutable reference to the pixel at the specified position.
    fn try_pixel_mut<'a>(&'a mut self, pos: Vector2<usize>)
        -> Result<Self::PixelMut<'a>, OutOfBounds>;

    /// Attempts to set the pixel at the specified position.
    fn try_set_pixel(&mut self, pos: Vector2<usize>, pixel: Self::PixelValue)
        -> Result<(), OutOfBounds>
    {
        *self.try_pixel_mut(pos)? = pixel;
        Ok(())
    }

    /// Attempts to get a mutable view of a region within the image.
    fn try_subimage_mut<'a>(&'a mut self, region: Rect<usize>)
        -> Result<SubimageMut<'a, Self>, OutOfBounds>
    {
        let region = self.check_pixel_region(region)?;
        Ok(SubimageMut { parent: self, region })
    }
}

/// Returned when attempting to access pixels outside of an image's boundaries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutOfBounds;

impl OutOfBounds {
    const MESSAGE: &'static str = "pixel/region out of bounds";
}

impl Display for OutOfBounds {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fmt.write_str(OutOfBounds::MESSAGE)
    }
}

impl Error for OutOfBounds {
    fn description(&self) -> &str { OutOfBounds::MESSAGE }
}
