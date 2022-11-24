/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::marker::PhantomData;
use std::ops::Deref;

use color::IntoColorLossy;
use math::{Rect, Vector2};

use crate::bitmap::Bitmap;
use crate::map::{Cloned, Convert, Copied, Map};
use crate::subimage::Subimage;
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

    /// Returns an image that converts this image into another color type.
    fn convert<'a, T>(&'a self) -> Convert<'a, T, Self::Pixel<'a>, Self>
    where
        Self::Pixel<'a>: IntoColorLossy<T>,
    {
        Convert { parent: self, _phantom: PhantomData }
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

    /// Gets the pixel at the specified position.
    fn get_pixel<'a>(&'a self, pos: Vector2<usize>) -> Self::Pixel<'a> {
        self.try_get_pixel(pos).unwrap()
    }

    /// Gets the pixel at the specified position without bounds checking.
    unsafe fn get_pixel_unchecked<'a>(&'a self, pos: Vector2<usize>) -> Self::Pixel<'a> {
        self.get_pixel(pos)
    }

    /// Returns the image's height in pixels.
    fn height(&self) -> usize { self.size().y }

    /// Uses a callback to map pixel values.
    fn map<'a, T: 'a, F: Fn(Self::Pixel<'a>) -> T>(&'a self, f: F) -> Map<'a, Self, T, F> {
        Map { callback: f, parent: self }
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

    /// Renders the image's contents to a `Bitmap`.
    fn to_bitmap<'a>(&'a self) -> Bitmap where Self: Image<Pixel<'a> = bool> {
        Bitmap::from(self)
    }

    /// Renders the image's contents to a `VecImage`.
    fn to_vec_image<'a>(&'a self) -> VecImage<Self::Pixel<'a>> {
        VecImage::from(self)
    }

    /// Attempts to get the pixel at the specified position.
    fn try_get_pixel<'a>(&'a self, pos: Vector2<usize>) -> Result<Self::Pixel<'a>, OutOfBounds>;

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

/// Trait for changing pixels in an image.
pub trait ImageMut: Image {
    /// Type used when changing a pixel. `Self::Pixel<'_>` should typically either be the same as
    /// this or a reference to it.
    type PixelValue;

    /// Sets the pixel at the specified location.
    fn set_pixel(&mut self, pos: Vector2<usize>, pixel: Self::PixelValue) {
        self.try_set_pixel(pos, pixel).unwrap()
    }

    /// Sets the pixel at the specified location without bounds checking.
    unsafe fn set_pixel_unchecked(&mut self, pos: Vector2<usize>, pixel: Self::PixelValue) {
        self.set_pixel(pos, pixel)
    }

    /// Attempts to set the pixel at the specified location.
    fn try_set_pixel(&mut self, pos: Vector2<usize>, pixel: Self::PixelValue)
        -> Result<(), OutOfBounds>;
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
