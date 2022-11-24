/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::num::FpCategory;

/// Trait for color components.
pub trait Component {
    /// Returns the maximum component value.
    ///
    /// Note that this may not necessarily be the greatest value that the type can represent. For
    /// integer types, this is the type's maximum value. For floating point types, this is `1.0`.
    fn max() -> Self;

    /// Returns the minimum component value.
    ///
    /// Note that this may not necessarily be the least value that the type can represent. For
    /// integer types, this is `0`. For floating point types, this is `0.0`.
    fn min() -> Self;

    /// Saturates the value between `min()` and `max()`.
    fn saturate(self) -> Self;

    /// Wraps the value between `min()` and `max()`.
    fn wrap(self) -> Self;
}

impl Component for bool {
    fn max() -> bool { true }
    fn min() -> bool { false }
    fn saturate(self) -> bool { self }
    fn wrap(self) -> bool { self }
}

/// Converts losslessly from another color component type.
pub trait FromComponent<T>: FromComponentLossy<T> {
    fn from_component(other: T) -> Self;
}

impl<T> FromComponent<T> for T {
    fn from_component(other: T) -> T { other }
}

impl<'a, T: Copy> FromComponent<&'a T> for T {
    fn from_component(other: &'a T) -> T { *other }
}

impl FromComponent<f32> for f64 {
    fn from_component(other: f32) -> f64 { other as f64 }
}

/// Converts losslessly into another color component type.
pub trait IntoComponent<T>: IntoComponentLossy<T> {
    fn into_component(self) -> T;
}

impl<F, T: FromComponent<F>> IntoComponent<T> for F {
    fn into_component(self) -> T { T::from_component(self) }
}

/// Converts lossily (or losslessly) from another color component type.
pub trait FromComponentLossy<T> {
    fn from_component_lossy(other: T) -> Self;
}

impl<T, F: IntoComponent<T>> FromComponentLossy<F> for T {
    fn from_component_lossy(other: F) -> T { other.into_component() }
}

impl FromComponentLossy<f64> for f32 {
    fn from_component_lossy(other: f64) -> f32 { other as f32 }
}

/// Converts lossily (or losslessly) into another color component type.
pub trait IntoComponentLossy<T> {
    fn into_component_lossy(self) -> T;
}

impl<F, T: FromComponentLossy<F>> IntoComponentLossy<T> for F {
    fn into_component_lossy(self) -> T { T::from_component_lossy(self) }
}

macro_rules! impl_uint {
    ($($type:ident),*) => { $(
        impl Component for $type {
            fn max() -> $type { $type::MAX }
            fn min() -> $type { 0 }
            fn saturate(self) -> $type { self }
            fn wrap(self) -> $type { self }
        }
    )* };
}

macro_rules! impl_float {
    ($($type:ident),*) => { $(
        impl Component for $type {
            fn max() -> $type { 1.0 }
            fn min() -> $type { 0.0 }

            fn saturate(self) -> $type {
                match self.classify() {
                    FpCategory::Nan => $type::NAN,
                    FpCategory::Zero => 0.0,
                    _ => if self < 0.0 { 0.0 } else if self >= 1.0 { 1.0 } else { self },
                }
            }

            fn wrap(self) -> $type {
                match self.classify() {
                    FpCategory::Nan => $type::NAN,
                    FpCategory::Zero => 0.0,
                    _ => if self < 0.0 { 1.0 + self.fract() } else { self.fract() },
                }
            }
        }
    )* };
}

macro_rules! impl_upscale_cast {
    { $($from:ident * $scale:tt -> $to:ident;)* } => { $(
        impl FromComponent<$from> for $to {
            fn from_component(other: $from) -> $to { other as $to * $scale }
        }

        impl<'a> FromComponent<&'a $from> for $to {
            fn from_component(other: &'a $from) -> $to { *other as $to * $scale }
        }
    )* };
}

macro_rules! impl_downscale_cast {
    { $($from:ident >> $shift:tt -> $to:ident;)* } => { $(
        impl FromComponentLossy<$from> for $to {
            fn from_component_lossy(other: $from) -> $to { (other >> $shift) as $to }
        }

        impl<'a> FromComponentLossy<&'a $from> for $to {
            fn from_component_lossy(other: &'a $from) -> $to { (*other >> $shift) as $to }
        }
    )* };
}

macro_rules! impl_int_to_float {
    { $($to:ident <- $($from:ident),*;)* } => { $( $(
        impl FromComponentLossy<$from> for $to {
            fn from_component_lossy(other: $from) -> $to { other as $to / $from::MAX as $to }
        }

        impl<'a> FromComponentLossy<&'a $from> for $to {
            fn from_component_lossy(other: &'a $from) -> $to { *other as $to / $from::MAX as $to }
        }
    )* )* };
}

macro_rules! impl_float_to_int {
    { $($from:ident -> $($to:ident),*;)* } => { $( $(
        impl FromComponentLossy<$from> for $to {
            fn from_component_lossy(other: $from) -> $to { (other * $to::MAX as $from) as $to }
        }

        impl<'a> FromComponentLossy<&'a $from> for $to {
            fn from_component_lossy(other: &'a $from) -> $to { (*other * $to::MAX as $from) as $to }
        }
    )* )* };
}

macro_rules! impl_from_to_bool {
    ($($ty:ty),*) => { $(
        impl FromComponent<bool> for $ty {
            fn from_component(other: bool) -> $ty {
                if other { Component::max() } else { Component::min() }
            }
        }

        impl FromComponentLossy<$ty> for bool {
            fn from_component_lossy(other: $ty) -> bool {
                other > <$ty as Component>::min()
            }
        }
    )* };
}

impl_uint!(u8, u16, u32, u64, u128);
impl_float!(f32, f64);

impl_upscale_cast! {
    u8 * 0x0101 -> u16;
    u8 * 0x01010101 -> u32;
    u8 * 0x01010101_01010101 -> u64;
    u8 * 0x01010101_01010101_01010101_01010101 -> u128;
    u16 * 0x00010001 -> u32;
    u16 * 0x00010001_00010001 -> u64;
    u16 * 0x00010001_00010001_00010001_00010001 -> u128;
    u32 * 0x00000001_00000001 -> u64;
    u32 * 0x00000001_00000001_00000001_00000001 -> u128;
    u64 * 0x00000000_00000001_00000000_00000001 -> u128;
}
impl_downscale_cast! {
    u16 >> 8 -> u8;
    u32 >> 24 -> u8;
    u32 >> 16 -> u16;
    u64 >> 56 -> u8;
    u64 >> 48 -> u16;
    u64 >> 32 -> u32;
    u128 >> 120 -> u8;
    u128 >> 112 -> u16;
    u128 >> 96 -> u32;
    u128 >> 64 -> u64;
}
impl_int_to_float! {
    f32 <- u8, u16, u32, u64, u128;
    f64 <- u8, u16, u32, u64, u128;
}
impl_float_to_int! {
    f32 -> u8, u16, u32, u64, u128;
    f64 -> u8, u16, u32, u64, u128;
}

impl_from_to_bool!(u8, u16, u32, u64, u128, f32, f64);
