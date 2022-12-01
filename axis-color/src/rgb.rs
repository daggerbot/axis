/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::color::{Color, FromColor, FromColorLossy};
use crate::component::{Component, IntoComponent, IntoComponentLossy};
use crate::lum::{Lum, LumAlpha};

/// Red color type.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Red<T> {
    pub r: T,
}

impl<T> Color for Red<T> {
    type Component = T;
    const NUM_COMPONENTS: usize = 1;
}

impl<T> FromColor<bool> for Red<T>
where
    bool: IntoComponent<T>,
{
    fn from_color(other: bool) -> Red<T> {
        Red {
            r: other.into_component(),
        }
    }
}

impl<T, F: IntoComponent<T>> FromColor<Lum<F>> for Red<T> {
    fn from_color(other: Lum<F>) -> Red<T> {
        Red {
            r: other.l.into_component(),
        }
    }
}

impl<T> FromColorLossy<bool> for Red<T>
where
    bool: IntoComponentLossy<T>,
{
    fn from_color_lossy(other: bool) -> Red<T> {
        Red {
            r: other.into_component_lossy(),
        }
    }
}

impl<T, F: IntoComponentLossy<T>> FromColorLossy<Lum<F>> for Red<T> {
    fn from_color_lossy(other: Lum<F>) -> Red<T> {
        Red {
            r: other.l.into_component_lossy(),
        }
    }
}

impl<T, F: IntoComponentLossy<T>> FromColorLossy<LumAlpha<F>> for Red<T> {
    fn from_color_lossy(other: LumAlpha<F>) -> Red<T> {
        Red {
            r: other.l.into_component_lossy(),
        }
    }
}

impl<T, F: IntoComponentLossy<T>> FromColorLossy<Rg<F>> for Red<T> {
    fn from_color_lossy(other: Rg<F>) -> Red<T> {
        Red {
            r: other.r.into_component_lossy(),
        }
    }
}

impl<T, F: IntoComponentLossy<T>> FromColorLossy<Rgb<F>> for Red<T> {
    fn from_color_lossy(other: Rgb<F>) -> Red<T> {
        Red {
            r: other.r.into_component_lossy(),
        }
    }
}

impl<T, F: IntoComponentLossy<T>> FromColorLossy<Rgba<F>> for Red<T> {
    fn from_color_lossy(other: Rgba<F>) -> Red<T> {
        Red {
            r: other.r.into_component_lossy(),
        }
    }
}

impl_scalar_ops! {
    impl Add::add for Red(r);
    impl Div::div for Red(r);
    impl Mul::mul for Red(r);
    impl Sub::sub for Red(r);
}

impl_scalar_assign_ops! {
    impl AddAssign::add_assign for Red(r);
    impl DivAssign::div_assign for Red(r);
    impl MulAssign::mul_assign for Red(r);
    impl SubAssign::sub_assign for Red(r);
}

/// Red-green color type.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rg<T> {
    pub r: T,
    pub g: T,
}

impl<T> Color for Rg<T> {
    type Component = T;
    const NUM_COMPONENTS: usize = 2;
}

impl<T: Clone, F: IntoComponent<T>> FromColor<Lum<F>> for Rg<T> {
    fn from_color(other: Lum<F>) -> Rg<T> {
        let l = other.l.into_component();
        Rg { r: l.clone(), g: l }
    }
}

impl<T: Component, F: IntoComponent<T>> FromColor<Red<F>> for Rg<T> {
    fn from_color(other: Red<F>) -> Rg<T> {
        Rg {
            r: other.r.into_component(),
            g: T::min(),
        }
    }
}

impl<T: Clone, F: IntoComponentLossy<T>> FromColorLossy<Lum<F>> for Rg<T> {
    fn from_color_lossy(other: Lum<F>) -> Rg<T> {
        let l = other.l.into_component_lossy();
        Rg { r: l.clone(), g: l }
    }
}

impl<T: Clone, F: IntoComponentLossy<T>> FromColorLossy<LumAlpha<F>> for Rg<T> {
    fn from_color_lossy(other: LumAlpha<F>) -> Rg<T> {
        let l = other.l.into_component_lossy();
        Rg { r: l.clone(), g: l }
    }
}

impl<T: Component, F: IntoComponentLossy<T>> FromColorLossy<Red<F>> for Rg<T> {
    fn from_color_lossy(other: Red<F>) -> Rg<T> {
        Rg {
            r: other.r.into_component_lossy(),
            g: T::min(),
        }
    }
}

impl<T, F: IntoComponentLossy<T>> FromColorLossy<Rgb<F>> for Rg<T> {
    fn from_color_lossy(other: Rgb<F>) -> Rg<T> {
        Rg {
            r: other.r.into_component_lossy(),
            g: other.g.into_component_lossy(),
        }
    }
}

impl<T, F: IntoComponentLossy<T>> FromColorLossy<Rgba<F>> for Rg<T> {
    fn from_color_lossy(other: Rgba<F>) -> Rg<T> {
        Rg {
            r: other.r.into_component_lossy(),
            g: other.g.into_component_lossy(),
        }
    }
}

impl<T> Into<(T, T)> for Rg<T> {
    fn into(self) -> (T, T) {
        (self.r, self.g)
    }
}

impl_scalar_ops! {
    impl Div::div for Rg(r, g);
    impl Mul::mul for Rg(r, g);
}

impl_scalar_assign_ops! {
    impl DivAssign::div_assign for Rg(r, g);
    impl MulAssign::mul_assign for Rg(r, g);
}

/// Red-green-blue color type.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rgb<T> {
    pub r: T,
    pub g: T,
    pub b: T,
}

impl<T> Color for Rgb<T> {
    type Component = T;
    const NUM_COMPONENTS: usize = 3;
}

impl<T: Clone, F: IntoComponent<T>> FromColor<Lum<F>> for Rgb<T> {
    fn from_color(other: Lum<F>) -> Rgb<T> {
        let l = other.l.into_component();
        Rgb {
            r: l.clone(),
            g: l.clone(),
            b: l,
        }
    }
}

impl<T: Component, F: IntoComponent<T>> FromColor<Red<F>> for Rgb<T> {
    fn from_color(other: Red<F>) -> Rgb<T> {
        Rgb {
            r: other.r.into_component(),
            g: T::min(),
            b: T::min(),
        }
    }
}

impl<T: Component, F: IntoComponent<T>> FromColor<Rg<F>> for Rgb<T> {
    fn from_color(other: Rg<F>) -> Rgb<T> {
        Rgb {
            r: other.r.into_component(),
            g: other.g.into_component(),
            b: T::min(),
        }
    }
}

impl<T: Clone, F: IntoComponentLossy<T>> FromColorLossy<Lum<F>> for Rgb<T> {
    fn from_color_lossy(other: Lum<F>) -> Rgb<T> {
        let l = other.l.into_component_lossy();
        Rgb {
            r: l.clone(),
            g: l.clone(),
            b: l,
        }
    }
}

impl<T: Clone, F: IntoComponentLossy<T>> FromColorLossy<LumAlpha<F>> for Rgb<T> {
    fn from_color_lossy(other: LumAlpha<F>) -> Rgb<T> {
        let l = other.l.into_component_lossy();
        Rgb {
            r: l.clone(),
            g: l.clone(),
            b: l,
        }
    }
}

impl<T: Component, F: IntoComponentLossy<T>> FromColorLossy<Red<F>> for Rgb<T> {
    fn from_color_lossy(other: Red<F>) -> Rgb<T> {
        Rgb {
            r: other.r.into_component_lossy(),
            g: T::min(),
            b: T::min(),
        }
    }
}

impl<T: Component, F: IntoComponentLossy<T>> FromColorLossy<Rg<F>> for Rgb<T> {
    fn from_color_lossy(other: Rg<F>) -> Rgb<T> {
        Rgb {
            r: other.r.into_component_lossy(),
            g: other.g.into_component_lossy(),
            b: T::min(),
        }
    }
}

impl<T, F: IntoComponentLossy<T>> FromColorLossy<Rgba<F>> for Rgb<T> {
    fn from_color_lossy(other: Rgba<F>) -> Rgb<T> {
        Rgb {
            r: other.r.into_component_lossy(),
            g: other.g.into_component_lossy(),
            b: other.b.into_component_lossy(),
        }
    }
}

impl<T> Into<(T, T, T)> for Rgb<T> {
    fn into(self) -> (T, T, T) {
        (self.r, self.g, self.b)
    }
}

impl_scalar_ops! {
    impl Div::div for Rgb(r, g, b);
    impl Mul::mul for Rgb(r, g, b);
}

impl_scalar_assign_ops! {
    impl DivAssign::div_assign for Rgb(r, g, b);
    impl MulAssign::mul_assign for Rgb(r, g, b);
}

/// Red-green-blue-alpha color type.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Rgba<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

impl<T> Color for Rgba<T> {
    type Component = T;
    const NUM_COMPONENTS: usize = 4;
}

impl<T: Clone + Component, F: IntoComponent<T>> FromColor<Lum<F>> for Rgba<T> {
    fn from_color(other: Lum<F>) -> Rgba<T> {
        let l = other.l.into_component();
        Rgba {
            r: l.clone(),
            g: l.clone(),
            b: l,
            a: T::max(),
        }
    }
}

impl<T: Clone, F: IntoComponent<T>> FromColor<LumAlpha<F>> for Rgba<T> {
    fn from_color(other: LumAlpha<F>) -> Rgba<T> {
        let l = other.l.into_component();
        Rgba {
            r: l.clone(),
            g: l.clone(),
            b: l,
            a: other.a.into_component(),
        }
    }
}

impl<T: Component, F: IntoComponent<T>> FromColor<Red<F>> for Rgba<T> {
    fn from_color(other: Red<F>) -> Rgba<T> {
        Rgba {
            r: other.r.into_component(),
            g: T::min(),
            b: T::min(),
            a: T::max(),
        }
    }
}

impl<T: Component, F: IntoComponent<T>> FromColor<Rg<F>> for Rgba<T> {
    fn from_color(other: Rg<F>) -> Rgba<T> {
        Rgba {
            r: other.r.into_component(),
            g: other.g.into_component(),
            b: T::min(),
            a: T::max(),
        }
    }
}

impl<T: Component, F: IntoComponent<T>> FromColor<Rgb<F>> for Rgba<T> {
    fn from_color(other: Rgb<F>) -> Rgba<T> {
        Rgba {
            r: other.r.into_component(),
            g: other.g.into_component(),
            b: other.b.into_component(),
            a: T::max(),
        }
    }
}

impl<T: Clone + Component, F: IntoComponentLossy<T>> FromColorLossy<Lum<F>> for Rgba<T> {
    fn from_color_lossy(other: Lum<F>) -> Rgba<T> {
        let l = other.l.into_component_lossy();
        Rgba {
            r: l.clone(),
            g: l.clone(),
            b: l,
            a: T::max(),
        }
    }
}

impl<T: Clone, F: IntoComponentLossy<T>> FromColorLossy<LumAlpha<F>> for Rgba<T> {
    fn from_color_lossy(other: LumAlpha<F>) -> Rgba<T> {
        let l = other.l.into_component_lossy();
        Rgba {
            r: l.clone(),
            g: l.clone(),
            b: l,
            a: other.a.into_component_lossy(),
        }
    }
}

impl<T: Component, F: IntoComponentLossy<T>> FromColorLossy<Red<F>> for Rgba<T> {
    fn from_color_lossy(other: Red<F>) -> Rgba<T> {
        Rgba {
            r: other.r.into_component_lossy(),
            g: T::min(),
            b: T::min(),
            a: T::max(),
        }
    }
}

impl<T: Component, F: IntoComponentLossy<T>> FromColorLossy<Rg<F>> for Rgba<T> {
    fn from_color_lossy(other: Rg<F>) -> Rgba<T> {
        Rgba {
            r: other.r.into_component_lossy(),
            g: other.g.into_component_lossy(),
            b: T::min(),
            a: T::max(),
        }
    }
}

impl<T: Component, F: IntoComponentLossy<T>> FromColorLossy<Rgb<F>> for Rgba<T> {
    fn from_color_lossy(other: Rgb<F>) -> Rgba<T> {
        Rgba {
            r: other.r.into_component_lossy(),
            g: other.g.into_component_lossy(),
            b: other.b.into_component_lossy(),
            a: T::max(),
        }
    }
}

impl<T> Into<(T, T, T, T)> for Rgba<T> {
    fn into(self) -> (T, T, T, T) {
        (self.r, self.g, self.b, self.a)
    }
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
    impl Red(r: T)[1];
    impl Rg(r: T, g: T)[2];
    impl Rgb(r: T, g: T, b: T)[3];
    impl Rgba(r: T, g: T, b: T, a: T)[4];
}
