/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#[cfg(feature = "byteorder")]
extern crate byteorder;
extern crate axis_color as color;
#[cfg(feature = "crc32fast")]
extern crate crc32fast;
#[cfg(feature = "flate2")]
extern crate flate2;
extern crate axis_math as math;

/// Support for image codecs as optional cargo features.
pub mod codec;

mod generate;
mod image;
mod map;
mod subimage;
#[allow(dead_code)]
mod util;
mod vec_image;

pub use generate::{Generate, blank, generate, solid};
pub use image::{Image, ImageExt, ImageMut, OutOfBounds};
pub use map::{Cloned, Copied, Map};
pub use subimage::{Subimage, SubimageMut};
pub use vec_image::VecImage;
