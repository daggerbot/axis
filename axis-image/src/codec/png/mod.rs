/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

mod chunk;
mod compress;
mod encode;
mod filter;
mod interlace;
mod pixel;

pub use self::chunk::{ChunkId, ChunkWriter, InvalidChunkId};
pub use self::compress::{CompressionMethod, InvalidCompressionMethod};
pub use self::encode::{
    write_IDAT, write_IEND, write_IHDR, write_PLTE, write_signature, Encoder, EncoderError,
};
pub use self::filter::{FilterMethod, InvalidFilterMethod};
pub use self::interlace::{InterlaceMethod, InvalidInterlaceMethod};
pub use self::pixel::{ColorType, InvalidBitDepth, InvalidColorType, Pixel, PixelComponent};

use math::Vector2;

use crate::image::Image;

/// PNG header data for the `IHDR` chunk.
#[derive(Clone, Copy, Debug)]
pub struct Header {
    pub bit_depth: u8,
    pub color_type: ColorType,
    pub compression_method: CompressionMethod,
    pub filter_method: FilterMethod,
    pub image_size: Vector2<usize>,
    pub interlace_method: Option<InterlaceMethod>,
}

impl Header {
    /// Gets the default PNG header for the specified image.
    pub fn for_image<'a, P: 'a + Pixel, I: Image<Pixel<'a> = P>>(image: &'a I) -> Header {
        Header {
            bit_depth: P::BIT_DEPTH,
            color_type: P::COLOR_TYPE,
            compression_method: CompressionMethod::Zlib,
            filter_method: FilterMethod::Base,
            image_size: image.size(),
            interlace_method: None,
        }
    }
}
