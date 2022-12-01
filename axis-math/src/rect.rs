/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::ops::Sub;

use crate::vector::Vector2;

/// Axis-aligned rectangle type defined as two points.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rect<T>(pub Vector2<T>, pub Vector2<T>);

impl<T> Rect<T> {
    /// Returns `&self.1.y - &self.0.y`.
    /// If the rectangle is not ordered (see `is_ordered()` and `is_partially_ordered()`, this value
    /// may be negative or underflow.
    pub fn height<'a>(&'a self) -> <&'a T as Sub>::Output
    where
        &'a T: Sub,
    {
        &self.1.y - &self.0.y
    }

    /// Determines whether the rectangle is *ordered* using `Ord`.
    /// A rectangle is *ordered* if `self.0.x <= self.1.x && self.0.y <= self.1.y`.
    /// That is, components on a given axis are in order.
    pub fn is_ordered(&self) -> bool
    where
        T: Ord,
    {
        self.is_partially_ordered()
    }

    /// Determines whether the rectangle is *ordered* using `PartialOrd`.
    /// A rectangle is *ordered* if `self.0.x <= self.1.x && self.0.y <= self.1.y`.
    /// That is, components on a given axis are in order.
    pub fn is_partially_ordered(&self) -> bool
    where
        T: PartialOrd,
    {
        self.0.x <= self.1.x && self.0.y <= self.1.y
    }

    /// Determines whether the rectangle is *positive* using `PartialOrd`.
    /// A rectangle is *positive* if `self.0.x < self.1.x && self.0.y < self.1.y`.
    /// That is, components are ordered and have a positive difference.
    pub fn is_partially_positive(&self) -> bool
    where
        T: PartialOrd,
    {
        self.0.x < self.1.x && self.0.y < self.1.y
    }

    /// Determines whether the rectangle is *positive* using `Ord`.
    /// A rectangle is *positive* if `self.0.x < self.1.x && self.0.y < self.1.y`.
    /// That is, components are ordered and have a positive difference.
    pub fn is_positive(&self) -> bool
    where
        T: Ord,
    {
        self.is_partially_ordered()
    }

    /// Constructs a `Rect` from its scalar parts.
    pub const fn new(x0: T, y0: T, x1: T, y1: T) -> Rect<T> {
        Rect(Vector2::new(x0, y0), Vector2::new(x1, y1))
    }

    /// Returns `Vector2::new(self.width(), self.height())`.
    /// If the rectangle is not ordered (see `is_ordered()` and `is_partially_ordered()`, these
    /// values may be negative or underflow.
    pub fn size<'a>(&'a self) -> Vector2<<&'a T as Sub>::Output>
    where
        &'a T: Sub,
    {
        Vector2::new(self.width(), self.height())
    }

    /// Returns `&self.1.x - &self.0.x`.
    /// If the rectangle is not ordered (see `is_ordered()` and `is_partially_ordered()`, this value
    /// may be negative or underflow.
    pub fn width<'a>(&'a self) -> <&'a T as Sub>::Output
    where
        &'a T: Sub,
    {
        &self.1.x - &self.0.x
    }
}
