/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use crate::error::{DivByZeroError, OverflowError, RangeError, UnderflowError};
use crate::num::IntLimits;

/// Attempts to add two values.
pub trait TryAdd<Rhs = Self> {
    type Output;
    type Error;
    fn try_add(self, rhs: Rhs) -> Result<Self::Output, Self::Error>;
}

/// Attempts to divide a value by another value.
pub trait TryDiv<Rhs = Self> {
    type Output;
    type Error;
    fn try_div(self, rhs: Rhs) -> Result<Self::Output, Self::Error>;
}

/// Attempts to multiply two values.
pub trait TryMul<Rhs = Self> {
    type Output;
    type Error;
    fn try_mul(self, rhs: Rhs) -> Result<Self::Output, Self::Error>;
}

/// Attempts to negate a value.
pub trait TryNeg {
    type Output;
    type Error;
    fn try_neg(self) -> Result<Self::Output, Self::Error>;
}

/// Attempts to subtract a value from another.
pub trait TrySub<Rhs = Self> {
    type Output;
    type Error;
    fn try_sub(self, rhs: Rhs) -> Result<Self::Output, Self::Error>;
}

/// Saturates the result of a checked arithmetic operation if an overflow/underflow occurs.
pub trait Saturate {
    type Output;
    fn saturate(self) -> Self::Output;
}

impl<T: IntLimits> Saturate for Result<T, RangeError> {
    type Output = T;

    fn saturate(self) -> T {
        match self {
            Ok(n) => n,
            Err(RangeError::Overflow) => T::max(),
            Err(RangeError::Underflow) => T::min(),
        }
    }
}

impl<T: IntLimits> Saturate for Result<T, OverflowError> {
    type Output = T;

    fn saturate(self) -> T {
        match self {
            Ok(n) => n,
            Err(_) => T::max(),
        }
    }
}

impl<T: IntLimits> Saturate for Result<T, UnderflowError> {
    type Output = T;

    fn saturate(self) -> T {
        match self {
            Ok(n) => n,
            Err(_) => T::min(),
        }
    }
}

//--------------------------------------------------------------------------------------------------

macro_rules! impl_unary_refs {
    { $(impl $trait:ident::$fn:ident for $type:ty;)* } => { $(
        impl<'a> $trait for &'a $type {
            type Output = $type;
            type Error = <$type as $trait>::Error;

            fn $fn(self) -> Result<$type, Self::Error> {
                $trait::$fn(*self)
            }
        }
    )* };
}

macro_rules! impl_binary_refs {
    { $(impl $trait:ident::$fn:ident for $type:ty;)* } => { $(
        impl<'a> $trait<$type> for &'a $type {
            type Output = $type;
            type Error = <$type as $trait>::Error;

            fn $fn(self, rhs: $type) -> Result<$type, Self::Error> {
                $trait::$fn(*self, rhs)
            }
        }

        impl<'r> $trait<&'r $type> for $type {
            type Output = $type;
            type Error = <$type as $trait>::Error;

            fn $fn(self, rhs: &'r $type) -> Result<$type, Self::Error> {
                $trait::$fn(self, *rhs)
            }
        }

        impl<'a, 'r> $trait<&'r $type> for &'a $type {
            type Output = $type;
            type Error = <$type as $trait>::Error;

            fn $fn(self, rhs: &'r $type) -> Result<$type, Self::Error> {
                $trait::$fn(*self, *rhs)
            }
        }
    )* };
}

/// Implements traits for all integer types.
macro_rules! impl_int {
    ($type:ident) => {
        impl TryDiv for $type {
            type Output = $type;
            type Error = DivByZeroError;

            fn try_div(self, rhs: $type) -> Result<$type, DivByZeroError> {
                match $type::checked_div(self, rhs) {
                    None => Err(DivByZeroError),
                    Some(n) => Ok(n),
                }
            }
        }

        impl_unary_refs! {
            impl TryNeg::try_neg for $type;
        }

        impl_binary_refs! {
            impl TryAdd::try_add for $type;
            impl TryDiv::try_div for $type;
            impl TryMul::try_mul for $type;
            impl TrySub::try_sub for $type;
        }
    };
}

/// Implements traits for signed integer types.
macro_rules! impl_sint {
    ($($type:ident),*) => { $(
        impl TryAdd for $type {
            type Output = $type;
            type Error = RangeError;

            fn try_add(self, rhs: $type) -> Result<$type, RangeError> {
                match $type::checked_add(self, rhs) {
                    None => {
                        Err(if self < 0 { RangeError::Underflow } else { RangeError::Overflow })
                    },
                    Some(n) => Ok(n),
                }
            }
        }

        impl TryMul for $type {
            type Output = $type;
            type Error = RangeError;

            fn try_mul(self, rhs: $type) -> Result<$type, RangeError> {
                match $type::checked_mul(self, rhs) {
                    None => Err(
                        if (self < 0) == (rhs < 0) {
                            RangeError::Overflow
                        } else {
                            RangeError::Underflow
                        }
                    ),
                    Some(n) => Ok(n),
                }
            }
        }

        impl TryNeg for $type {
            type Output = $type;
            type Error = OverflowError;

            fn try_neg(self) -> Result<$type, OverflowError> {
                match $type::checked_neg(self) {
                    None => Err(OverflowError),
                    Some(n) => Ok(n),
                }
            }
        }

        impl TrySub for $type {
            type Output = $type;
            type Error = RangeError;

            fn try_sub(self, rhs: $type) -> Result<$type, RangeError> {
                match $type::checked_sub(self, rhs) {
                    None => {
                        Err(if self < 0 { RangeError::Underflow } else { RangeError::Overflow })
                    },
                    Some(n) => Ok(n),
                }
            }
        }

        impl_int!($type);
    )* };
}

/// Implements traits for unsigned integer types.
macro_rules! impl_uint {
    ($($type:ident),*) => { $(
        impl TryAdd for $type {
            type Output = $type;
            type Error = OverflowError;

            fn try_add(self, rhs: $type) -> Result<$type, OverflowError> {
                match $type::checked_add(self, rhs) {
                    None => Err(OverflowError),
                    Some(n) => Ok(n),
                }
            }
        }

        impl TryMul for $type {
            type Output = $type;
            type Error = OverflowError;

            fn try_mul(self, rhs: $type) -> Result<$type, OverflowError> {
                match $type::checked_mul(self, rhs) {
                    None => Err(OverflowError),
                    Some(n) => Ok(n),
                }
            }
        }

        impl TryNeg for $type {
            type Output = $type;
            type Error = UnderflowError;

            fn try_neg(self) -> Result<$type, UnderflowError> {
                match $type::checked_neg(self) {
                    None => Err(UnderflowError),
                    Some(n) => Ok(n),
                }
            }
        }

        impl TrySub for $type {
            type Output = $type;
            type Error = UnderflowError;

            fn try_sub(self, rhs: $type) -> Result<$type, UnderflowError> {
                match $type::checked_sub(self, rhs) {
                    None => Err(UnderflowError),
                    Some(n) => Ok(n),
                }
            }
        }

        impl_int!($type);
    )* };
}

impl_sint!(i8, i16, i32, i64, i128, isize);
impl_uint!(u8, u16, u32, u64, u128, usize);
