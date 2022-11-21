/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/// Color type properties.
pub trait Color {
    type Component;
    const NUM_COMPONENTS: usize;
}

/// Trait for conversion from another color type.
pub trait FromColor<T> : FromColorLossy<T> {
    fn from_color(other: T) -> Self;
}

/// Trait for conversion into another color type.
pub trait IntoColor<T> : IntoColorLossy<T> {
    fn into_color(self) -> T;
}

impl<F, T: FromColor<F>> IntoColor<T> for F {
    fn into_color(self) -> T { T::from_color(self) }
}

/// Trait for lossy conversion into another color type.
pub trait FromColorLossy<T> {
    fn from_color_lossy(other: T) -> Self;
}

/// Trait for lossy conversion into another color type.
pub trait IntoColorLossy<T> {
    fn into_color_lossy(self) -> T;
}

impl<F, T: FromColorLossy<F>> IntoColorLossy<T> for F {
    fn into_color_lossy(self) -> T { T::from_color_lossy(self) }
}
