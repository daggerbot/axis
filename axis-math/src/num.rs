/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::ops::{Add, Div, Mul, Sub};

/// Trait for scalar types that can be treated as continuous (computers cannot represent true
/// continuous values).
pub trait Continuous : Scalar {}
impl Continuous for f32 {}
impl Continuous for f64 {}

/// Gets a type's multiplicative identity.
pub trait Identity {
    fn identity() -> Self;
}

/// Gets the minimum and maximum values for an integer type.
pub trait IntLimits {
    fn min() -> Self;
    fn max() -> Self;
}

/// Trait for types that exhibit common properties of scalars.
pub trait Scalar : Add + Div + Mul + PartialEq + Sized + Sub {}
impl<T: Add + Div + Mul + PartialEq + Sized + Sub> Scalar for T {}

/// Gets a type's additive identity.
pub trait Zero {
    fn zero() -> Self;
}

macro_rules! impl_int {
    ($($type:ident),*) => { $(
        impl Identity for $type {
            fn identity() -> $type { 1 }
        }

        impl IntLimits for $type {
            fn min() -> $type { $type::MIN }
            fn max() -> $type { $type::MAX }
        }

        impl Zero for $type {
            fn zero() -> $type { 0 }
        }
    )* };
}

macro_rules! impl_uint {
    ($($type:ident),*) => { $(
        impl Identity for $type {
            fn identity() -> $type { 1 }
        }

        impl IntLimits for $type {
            fn min() -> $type { 0 }
            fn max() -> $type { $type::MAX }
        }

        impl Zero for $type {
            fn zero() -> $type { 0 }
        }
    )* };
}

macro_rules! impl_float {
    ($($type:ident),*) => { $(
        impl Identity for $type {
            fn identity() -> $type { 1.0 }
        }

        impl Zero for $type {
            fn zero() -> $type { 0.0 }
        }
    )* };
}

impl_int!(i8, i16, i32, i64, i128, isize);
impl_uint!(u8, u16, u32, u64, u128, usize);
impl_float!(f32, f64);
