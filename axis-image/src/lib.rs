/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

extern crate axis_color as color;
extern crate axis_math as math;
#[cfg(feature = "byteorder")]
extern crate byteorder;
#[cfg(feature = "crc32fast")]
extern crate crc32fast;
#[cfg(feature = "flate2")]
extern crate flate2;

/// Support for image codecs as optional cargo features.
pub mod codec;

mod bitmap;
mod generate;
mod image;
mod map;
mod subimage;
#[allow(dead_code)]
mod util;
mod vec_image;

pub use bitmap::Bitmap;
pub use generate::{blank, generate, solid, Generate};
pub use image::{Image, ImageMut, OutOfBounds};
pub use map::{Cloned, Convert, Copied, Map};
pub use subimage::{Subimage, SubimageMut};
pub use vec_image::VecImage;
