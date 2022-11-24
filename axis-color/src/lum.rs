/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::color::{Color, FromColor, FromColorLossy};
use crate::component::{IntoComponent, IntoComponentLossy};

/// Luminance color type.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct Lum<T> {
    pub l: T,
}

impl<T> Color for Lum<T> {
    type Component = T;
    const NUM_COMPONENTS: usize = 1;
}

impl<T> FromColor<bool> for Lum<T> where bool: IntoComponent<T> {
    fn from_color(other: bool) -> Lum<T> {
        Lum { l: other.into_component() }
    }
}

impl<T> FromColorLossy<bool> for Lum<T> where bool: IntoComponentLossy<T> {
    fn from_color_lossy(other: bool) -> Lum<T> {
        Lum { l: other.into_component_lossy() }
    }
}

impl_scalar_ops! {
    impl Add::add for Lum(l);
    impl Div::div for Lum(l);
    impl Mul::mul for Lum(l);
    impl Sub::sub for Lum(l);
}

impl_scalar_assign_ops! {
    impl AddAssign::add_assign for Lum(l);
    impl DivAssign::div_assign for Lum(l);
    impl MulAssign::mul_assign for Lum(l);
    impl SubAssign::sub_assign for Lum(l);
}

/// Luminance-alpha color type.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LumAlpha<T> {
    pub l: T,
    pub a: T,
}

impl<T> Color for LumAlpha<T> {
    type Component = T;
    const NUM_COMPONENTS: usize = 2;
}

macro_rules! impl_all {
    { $(impl $color:ident($($field:ident: $T:ident),*)[$n:expr];)* } => { $(
        impl<T> $color<T> {
            /// Constructs a new color from its components.
            pub const fn new($($field: $T),*) -> $color<T> {
                $color { $($field),* }
            }
        }

        impl<T> From<[T; $n]> for $color<T> {
            fn from(a: [T; $n]) -> $color<T> {
                let mut iter = a.into_iter();
                $(let $field = iter.next().unwrap();)*
                $color { $($field),* }
            }
        }

        #[allow(unused_parens)]
        impl<T> From<($($T),*)> for $color<T> {
            fn from(t: ($($T),*)) -> $color<T> {
                let ($($field),*) = t;
                $color { $($field),* }
            }
        }

        impl<T, F: IntoComponent<T>> FromColor<$color<F>> for $color<T> {
            fn from_color(other: $color<F>) -> $color<T> {
                $color { $($field: other.$field.into_component()),* }
            }
        }

        impl<T, F: IntoComponentLossy<T>> FromColorLossy<$color<F>> for $color<T> {
            fn from_color_lossy(other: $color<F>) -> $color<T> {
                $color { $($field: other.$field.into_component_lossy()),* }
            }
        }

        impl<T> Into<[T; $n]> for $color<T> {
            fn into(self) -> [T; $n] {
                [$(self.$field),*]
            }
        }

        impl<T> IntoIterator for $color<T> {
            type IntoIter = std::array::IntoIter<T, $n>;
            type Item = T;

            fn into_iter(self) -> Self::IntoIter {
                <Self as Into<[T; $n]>>::into(self).into_iter()
            }
        }
    )* };
}

impl_all! {
    impl Lum(l: T)[1];
    impl LumAlpha(l: T, a: T)[2];
}
