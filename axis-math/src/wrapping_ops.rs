/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/// Adds two values and wraps the result on overflow/underflow.
pub trait WrappingAdd<Rhs = Self> {
    type Output;
    fn wrapping_add(self, rhs: Rhs) -> Self::Output;
}

/// Multiplies two values and wraps the result on overflow/underflow.
pub trait WrappingMul<Rhs = Self> {
    type Output;
    fn wrapping_mul(self, rhs: Rhs) -> Self::Output;
}

/// Subtracts a value from another and wraps the result on overflow/underflow.
pub trait WrappingSub<Rhs = Self> {
    type Output;
    fn wrapping_sub(self, rhs: Rhs) -> Self::Output;
}

macro_rules! impl_binary {
    { $(impl $trait:ident::$fn:ident for $type:ident;)* } => { $(
        impl $trait for $type {
            type Output = $type;
            fn $fn(self, rhs: $type) -> $type { $type::$fn(self, rhs) }
        }

        impl<'a> $trait<$type> for &'a $type {
            type Output = $type;
            fn $fn(self, rhs: $type) -> $type { $type::$fn(*self, rhs) }
        }

        impl<'r> $trait<&'r $type> for $type {
            type Output = $type;
            fn $fn(self, rhs: &'r $type) -> $type { $type::$fn(self, *rhs) }
        }

        impl<'a, 'r> $trait<&'r $type> for &'a $type {
            type Output = $type;
            fn $fn(self, rhs: &'r $type) -> $type { $type::$fn(*self, *rhs) }
        }
    )* };
}

macro_rules! impl_all {
    ($($type:ident),*) => { $(
        impl_binary! {
            impl WrappingAdd::wrapping_add for $type;
            impl WrappingMul::wrapping_mul for $type;
            impl WrappingSub::wrapping_sub for $type;
        }
    )* };
}

impl_all!(i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize);
