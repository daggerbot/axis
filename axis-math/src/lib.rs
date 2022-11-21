/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;
#[cfg(not(feature = "std"))]
extern crate core as std;

mod convert;
mod error;
mod num;
mod rect;
mod try_ops;
mod vector;
mod wrapping_ops;

pub use convert::{TryFromComposite, TryIntoComposite};
pub use error::{DivByZeroError, OverflowError, RangeError, UnderflowError};
pub use num::{Continuous, Identity, IntLimits, Scalar, Zero};
pub use rect::Rect;
pub use try_ops::{Saturate, TryAdd, TryDiv, TryMul, TryNeg, TrySub};
pub use vector::{Vector2, Vector3, Vector4};
pub use wrapping_ops::{WrappingAdd, WrappingMul, WrappingSub};
