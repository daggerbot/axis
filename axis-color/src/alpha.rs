/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::color::{Color, FromColor, FromColorLossy};
use crate::component::{IntoComponent, IntoComponentLossy};
use crate::rgb::Rgba;

/// Alpha color type.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct Alpha<T> {
    pub a: T,
}

impl<T> Alpha<T> {
    /// Constructs a new color from its components.
    pub const fn new(a: T) -> Alpha<T> {
        Alpha { a }
    }
}

impl<T> Color for Alpha<T> {
    type Component = T;
    const NUM_COMPONENTS: usize = 1;
}

impl<T> From<T> for Alpha<T> {
    fn from(a: T) -> Alpha<T> {
        Alpha { a }
    }
}

impl<T> From<[T; 1]> for Alpha<T> {
    fn from(a: [T; 1]) -> Alpha<T> {
        Alpha { a: a.into_iter().next().unwrap() }
    }
}

impl<T, F: IntoComponent<T>> FromColor<Alpha<F>> for Alpha<T> {
    fn from_color(other: Alpha<F>) -> Alpha<T> {
        Alpha { a: other.a.into_component() }
    }
}

impl<T, F: IntoComponentLossy<T>> FromColorLossy<Alpha<F>> for Alpha<T> {
    fn from_color_lossy(other: Alpha<F>) -> Alpha<T> {
        Alpha { a: other.a.into_component_lossy() }
    }
}

impl<T, F: IntoComponentLossy<T>> FromColorLossy<Rgba<F>> for Alpha<T> {
    fn from_color_lossy(other: Rgba<F>) -> Alpha<T> {
        Alpha { a: other.a.into_component_lossy() }
    }
}

impl<T> Into<[T; 1]> for Alpha<T> {
    fn into(self) -> [T; 1] { [self.a] }
}

impl<T> IntoIterator for Alpha<T> {
    type IntoIter = std::array::IntoIter<T, 1>;
    type Item = T;

    fn into_iter(self) -> std::array::IntoIter<T, 1> {
        <Self as Into<[T; 1]>>::into(self).into_iter()
    }
}
