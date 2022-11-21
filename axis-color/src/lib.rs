/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

extern crate axis_math as math;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;
extern crate core as std;

mod alpha;
mod color;
mod component;
mod lum;
mod predefined;
mod rgb;

pub use alpha::Alpha;
pub use color::{Color, FromColor, FromColorLossy, IntoColor, IntoColorLossy};
pub use component::{
    Component,
    FromComponent,
    FromComponentLossy,
    IntoComponent,
    IntoComponentLossy,
};
pub use lum::{Lum, LumAlpha};
pub use predefined::*;
pub use rgb::{Red, Rg, Rgb, Rgba};
