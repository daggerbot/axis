/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

/// Converts to another type, typically by using the `as` operator.
pub trait FromLossy<F>: Sized {
    fn from_lossy(from: F) -> Self;
}

impl<T> FromLossy<T> for T {
    fn from_lossy(from: T) -> T {
        from
    }
}

macro_rules! impl_from_lossy {
    { $($from:ty => $($to:ty),*;)* } => { $( $(
        impl FromLossy<$from> for $to {
            fn from_lossy(from: $from) -> $to { from as $to }
        }
    )* )* };
}

impl_from_lossy! {
    i8 => i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64;
    i16 => i8, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64;
    i32 => i8, i16, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64;
    i64 => i8, i16, i32, i128, isize, u8, u16, u32, u64, u128, usize, f32, f64;
    i128 => i8, i16, i32, i64, isize, u8, u16, u32, u64, u128, usize, f32, f64;
    isize => i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize, f32, f64;
    u8 => i8, i16, i32, i64, i128, isize, u16, u32, u64, u128, usize, f32, f64;
    u16 => i8, i16, i32, i64, i128, isize, u8, u32, u64, u128, usize, f32, f64;
    u32 => i8, i16, i32, i64, i128, isize, u8, u16, u64, u128, usize, f32, f64;
    u64 => i8, i16, i32, i64, i128, isize, u8, u16, u32, u128, usize, f32, f64;
    u128 => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, usize, f32, f64;
    usize => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, f32, f64;
    f32 => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f64;
    f64 => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize, f32;
}

/// Converts to another type, typically by using the `as` operator.
pub trait IntoLossy<T>: Sized {
    fn into_lossy(self) -> T;
}

impl<F, T: FromLossy<F>> IntoLossy<T> for F {
    fn into_lossy(self) -> T {
        T::from_lossy(self)
    }
}

/// Converts between composite types consisting of multiple scalars.
///
/// This is used instead of `From` to avoid conflicts with blanket implementations.
pub trait FromComposite<F>: Sized {
    fn from_composite(from: F) -> Self;
}

/// Converts between composite types consisting of multiple scalars.
///
/// This is used instead of `Into` to avoid conflicts with blanket implementations.
pub trait IntoComposite<T>: Sized {
    fn into_composite(self) -> T;
}

impl<F, T: FromComposite<F>> IntoComposite<T> for F {
    fn into_composite(self) -> T {
        T::from_composite(self)
    }
}

/// Converts between composite types, typically by using the `as` operator for each scalar.
///
/// This is used instead of `FromLossy` to avoid conflicts with blanket implementations.
pub trait FromCompositeLossy<F>: Sized {
    fn from_composite_lossy(from: F) -> Self;
}

/// Converts between composite types, typically by using the `as` operator for each scalar.
///
/// This is used instead of `IntoLossy` to avoid conflicts with blanket implementations.
pub trait IntoCompositeLossy<T>: Sized {
    fn into_composite_lossy(self) -> T;
}

impl<F, T: FromCompositeLossy<F>> IntoCompositeLossy<T> for F {
    fn into_composite_lossy(self) -> T {
        T::from_composite_lossy(self)
    }
}

/// Attempts to convert composite numeric types.
///
/// This is used instead of `TryFrom` to avoid conflicts with blanket implementations.
pub trait TryFromComposite<F>: Sized {
    type Error: Sized;
    fn try_from_composite(from: F) -> Result<Self, Self::Error>;
}

/// Attempts to convert composite numeric types.
///
/// This is used instead of `TryInto` to avoid conflicts with blanket implementations.
pub trait TryIntoComposite<T>: Sized {
    type Error: Sized;
    fn try_into_composite(self) -> Result<T, Self::Error>;
}

impl<F, T: TryFromComposite<F>> TryIntoComposite<T> for F {
    type Error = T::Error;

    fn try_into_composite(self) -> Result<T, Self::Error> {
        T::try_from_composite(self)
    }
}
