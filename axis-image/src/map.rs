/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::marker::PhantomData;
use std::ops::Deref;

use color::{Color, IntoColor, IntoColorLossy, IntoComponent, WithAlpha};
use math::Vector2;

use crate::image::{Image, OutOfBounds};

/// Invokes a callback with the corresponding pixel from an existing image for each pixel requested.
pub struct Map<'a, I: 'a + ?Sized + Image, T: 'a, F: Fn(I::Pixel<'a>) -> T> {
    pub(crate) callback: F,
    pub(crate) parent: &'a I,
}

impl<'a, I: 'a + Image, T: 'a, F: Fn(I::Pixel<'a>) -> T> Image for Map<'a, I, T, F> {
    type Pixel<'b> = T where Self: 'b;

    unsafe fn get_pixel_unchecked<'b>(&'b self, pos: Vector2<usize>) -> T {
        (self.callback)(self.parent.get_pixel_unchecked(pos))
    }

    fn height(&self) -> usize {
        self.parent.height()
    }

    fn size(&self) -> Vector2<usize> {
        self.parent.size()
    }

    fn try_get_pixel<'b>(&'b self, pos: Vector2<usize>) -> Result<T, OutOfBounds> {
        Ok((self.callback)(self.parent.try_get_pixel(pos)?))
    }

    fn width(&self) -> usize {
        self.parent.width()
    }
}

/// Clones the values referenced by the parent image.
pub struct Cloned<'a, T: 'a + Clone, I: 'a + ?Sized + Image>
where
    <I as Image>::Pixel<'a>: Deref<Target = T>,
{
    pub(crate) parent: &'a I,
}

impl<'a, T: 'a + Clone, I: 'a + Image> Image for Cloned<'a, T, I>
where
    <I as Image>::Pixel<'a>: Deref<Target = T>,
{
    type Pixel<'b> = T where Self: 'b;

    unsafe fn get_pixel_unchecked<'b>(&'b self, pos: Vector2<usize>) -> T {
        self.parent.get_pixel_unchecked(pos).clone()
    }

    fn height(&self) -> usize {
        self.parent.height()
    }

    fn size(&self) -> Vector2<usize> {
        self.parent.size()
    }

    fn try_get_pixel<'b>(&'b self, pos: Vector2<usize>) -> Result<T, OutOfBounds> {
        Ok(self.parent.try_get_pixel(pos)?.clone())
    }

    fn width(&self) -> usize {
        self.parent.width()
    }
}

/// Converts requested pixels to another color type.
pub struct Convert<'a, T, F: 'a + IntoColor<T>, I: 'a + ?Sized + Image<Pixel<'a> = F>> {
    pub(crate) parent: &'a I,
    pub(crate) _phantom: PhantomData<T>,
}

impl<'a, T, F: 'a + IntoColor<T>, I: 'a + Image<Pixel<'a> = F>> Image for Convert<'a, T, F, I> {
    type Pixel<'b> = T where Self: 'b;

    unsafe fn get_pixel_unchecked<'b>(&'b self, pos: Vector2<usize>) -> T {
        self.parent.get_pixel_unchecked(pos).into_color()
    }

    fn height(&self) -> usize {
        self.parent.height()
    }

    fn size(&self) -> Vector2<usize> {
        self.parent.size()
    }

    fn try_get_pixel<'b>(&'b self, pos: Vector2<usize>) -> Result<T, OutOfBounds> {
        Ok(self.parent.try_get_pixel(pos)?.into_color())
    }

    fn width(&self) -> usize {
        self.parent.width()
    }
}

/// Converts requested pixels to another color type.
pub struct ConvertLossy<'a, T, F: 'a + IntoColorLossy<T>, I: 'a + ?Sized + Image<Pixel<'a> = F>> {
    pub(crate) parent: &'a I,
    pub(crate) _phantom: PhantomData<T>,
}

impl<'a, T, F: 'a + IntoColorLossy<T>, I: 'a + Image<Pixel<'a> = F>> Image
    for ConvertLossy<'a, T, F, I>
{
    type Pixel<'b> = T where Self: 'b;

    unsafe fn get_pixel_unchecked<'b>(&'b self, pos: Vector2<usize>) -> T {
        self.parent.get_pixel_unchecked(pos).into_color_lossy()
    }

    fn height(&self) -> usize {
        self.parent.height()
    }

    fn size(&self) -> Vector2<usize> {
        self.parent.size()
    }

    fn try_get_pixel<'b>(&'b self, pos: Vector2<usize>) -> Result<T, OutOfBounds> {
        Ok(self.parent.try_get_pixel(pos)?.into_color_lossy())
    }

    fn width(&self) -> usize {
        self.parent.width()
    }
}

/// Copies the values referenced by the parent image.
pub struct Copied<'a, T: 'a + Copy, I: 'a + ?Sized + Image>
where
    <I as Image>::Pixel<'a>: Deref<Target = T>,
{
    pub(crate) parent: &'a I,
}

impl<'a, T: 'a + Copy, I: 'a + Image> Image for Copied<'a, T, I>
where
    <I as Image>::Pixel<'a>: Deref<Target = T>,
{
    type Pixel<'b> = T where Self: 'b;

    unsafe fn get_pixel_unchecked<'b>(&'b self, pos: Vector2<usize>) -> T {
        *self.parent.get_pixel_unchecked(pos)
    }

    fn height(&self) -> usize {
        self.parent.height()
    }

    fn size(&self) -> Vector2<usize> {
        self.parent.size()
    }

    fn try_get_pixel<'b>(&'b self, pos: Vector2<usize>) -> Result<T, OutOfBounds> {
        Ok(*self.parent.try_get_pixel(pos)?)
    }

    fn width(&self) -> usize {
        self.parent.width()
    }
}

/// Image which converts pixels from another image using the `Into` trait.
pub struct To<'a, T, F: 'a + Into<T>, I: 'a + ?Sized + Image<Pixel<'a> = F>> {
    pub(crate) parent: &'a I,
    pub(crate) _phantom: PhantomData<T>,
}

impl<'a, T, F: 'a + Into<T>, I: 'a + Image<Pixel<'a> = F>> Image for To<'a, T, F, I> {
    type Pixel<'b> = T where Self: 'b;

    unsafe fn get_pixel_unchecked<'b>(&'b self, pos: Vector2<usize>) -> T {
        self.parent.get_pixel_unchecked(pos).into()
    }

    fn height(&self) -> usize {
        self.parent.height()
    }

    fn size(&self) -> Vector2<usize> {
        self.parent.size()
    }

    fn try_get_pixel<'b>(&'b self, pos: Vector2<usize>) -> Result<T, OutOfBounds> {
        Ok(self.parent.try_get_pixel(pos)?.into())
    }

    fn width(&self) -> usize {
        self.parent.width()
    }
}

/// Image which adds an alpha channel to another image.
pub struct WithMask<'a, I: 'a + ?Sized + Image, M: Image>
where
    I::Pixel<'a>: WithAlpha,
    for<'b> M::Pixel<'b>: IntoComponent<<I::Pixel<'a> as Color>::Component>,
{
    pub(crate) parent: &'a I,
    pub(crate) mask: M,
}

impl<'a, I: 'a + Image, M: Image> Image for WithMask<'a, I, M>
where
    I::Pixel<'a>: WithAlpha,
    for<'b> M::Pixel<'b>: IntoComponent<<I::Pixel<'a> as Color>::Component>,
{
    type Pixel<'b> = <I::Pixel<'a> as WithAlpha>::Output where Self: 'b;

    unsafe fn get_pixel_unchecked<'b>(&'b self, pos: Vector2<usize>) -> Self::Pixel<'b> {
        self.parent.get_pixel_unchecked(pos)
                   .with_alpha(self.mask.get_pixel_unchecked(pos).into_component())
    }

    fn height(&self) -> usize {
        self.parent.height()
    }

    fn size(&self) -> Vector2<usize> {
        self.parent.size()
    }

    fn try_get_pixel<'b>(&'b self, pos: Vector2<usize>) -> Result<Self::Pixel<'b>, OutOfBounds> {
        Ok(self.parent.try_get_pixel(pos)?
                      .with_alpha(self.mask.try_get_pixel(pos)?.into_component()))
    }

    fn width(&self) -> usize {
        self.parent.width()
    }
}

/// Image which yields tuples containing pixels from two different images.
pub struct Zip<'a, A: 'a + ?Sized + Image, B: Image>(pub(crate) &'a A, pub(crate) B);

impl<'a, A: 'a + Image, B: Image> Image for Zip<'a, A, B> {
    type Pixel<'b> = (A::Pixel<'b>, B::Pixel<'b>) where Self: 'b;

    unsafe fn get_pixel_unchecked<'b>(&'b self, pos: Vector2<usize>) -> Self::Pixel<'b> {
        (self.0.get_pixel_unchecked(pos), self.1.get_pixel_unchecked(pos))
    }

    fn height(&self) -> usize {
        self.0.height()
    }

    fn size(&self) -> Vector2<usize> {
        self.0.size()
    }

    fn try_get_pixel<'b>(&'b self, pos: Vector2<usize>) -> Result<Self::Pixel<'b>, OutOfBounds> {
        Ok((self.0.try_get_pixel(pos)?, self.1.try_get_pixel(pos)?))
    }

    fn width(&self) -> usize {
        self.0.width()
    }
}
