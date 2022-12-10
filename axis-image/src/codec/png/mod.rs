/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

mod chunk;
mod compress;
mod decode;
mod encode;
mod filter;
mod interlace;

pub use self::chunk::{ChunkId, ChunkReader, ChunkWriter, ProgressiveChunkReader};
pub use self::compress::CompressionMethod;
pub use self::decode::{
    read, read_IDAT, read_IHDR, read_PLTE, read_file, read_signature, AnyPixelReader, DecodePixel,
    DecodedImage, PixelReader,
};
pub use self::encode::{
    write_IDAT, write_IEND, write_IHDR, write_PLTE, write_signature, EncodePixel, Encoder,
};
pub use self::filter::FilterMethod;
pub use self::interlace::InterlaceMethod;

use std::fmt::{Display, Formatter};

use color::{FromComponentLossy, IntoComponentLossy, Lum, LumAlpha, Rgb, Rgba};
use math::{FromLossy, IntoLossy, Vector2};

use crate::image::Image;

const IHDR_LENGTH: u32 = 13;
const MAX_DIMENSION: u32 = 0x7fff_ffff;
const MAX_PALETTE_LEN: usize = 256;

/// Enumeration of PNG color types.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum ColorType {
    Gray = 0,
    Rgb = 2,
    Index = 3,
    GrayAlpha = 4,
    RgbAlpha = 6,
}

impl ColorType {
    /// Returns the number of channels in each color.
    pub const fn channel_count(self) -> usize {
        match self {
            ColorType::Gray => 1,
            ColorType::Rgb => 3,
            ColorType::Index => 1,
            ColorType::GrayAlpha => 2,
            ColorType::RgbAlpha => 4,
        }
    }

    /// Determines whether the specified bit depth is allowed for this color type.
    pub fn check_bit_depth(self, bit_depth: u8) -> Result<u8, Error> {
        match self {
            ColorType::Gray => match bit_depth {
                1 | 2 | 4 | 8 | 16 => return Ok(bit_depth),
                _ => (),
            },
            ColorType::Rgb => match bit_depth {
                8 | 16 => return Ok(bit_depth),
                _ => (),
            },
            ColorType::Index => match bit_depth {
                1 | 2 | 4 | 8 => return Ok(bit_depth),
                _ => (),
            },
            ColorType::GrayAlpha => match bit_depth {
                8 | 16 => return Ok(bit_depth),
                _ => (),
            },
            ColorType::RgbAlpha => match bit_depth {
                8 | 16 => return Ok(bit_depth),
                _ => (),
            },
        }

        Err(Error::BitDepth {
            bit_depth,
            color_type: self,
        })
    }
}

impl ColorType {
    const fn description(self) -> &'static str {
        match self {
            ColorType::Gray => "gray",
            ColorType::Rgb => "rgb",
            ColorType::Index => "index",
            ColorType::GrayAlpha => "gray alpha",
            ColorType::RgbAlpha => "rgb alpha",
        }
    }
}

impl Display for ColorType {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fmt.write_str(self.description())
    }
}

impl TryFrom<u8> for ColorType {
    type Error = Error;

    fn try_from(raw: u8) -> Result<ColorType, Error> {
        match raw {
            0 => Ok(ColorType::Gray),
            2 => Ok(ColorType::Rgb),
            3 => Ok(ColorType::Index),
            4 => Ok(ColorType::GrayAlpha),
            6 => Ok(ColorType::RgbAlpha),
            _ => Err(Error::ColorType { raw }),
        }
    }
}

/// Trait for PNG pixel components.
pub trait PixelComponent:
    Copy
    + FromComponentLossy<u8>
    + FromComponentLossy<u16>
    + FromLossy<u8>
    + FromLossy<u16>
    + IntoComponentLossy<u8>
    + IntoComponentLossy<u16>
    + IntoLossy<u8>
    + IntoLossy<u16>
{
    /// Default bit depth.
    const BIT_DEPTH: u8;
}

impl PixelComponent for u8 {
    const BIT_DEPTH: u8 = 8;
}

impl PixelComponent for u16 {
    const BIT_DEPTH: u8 = 16;
}

/// Trait for PNG pixel types.
pub trait Pixel: Copy {
    /// Pixel component type.
    type Component: PixelComponent;

    /// Default bit depth.
    const BIT_DEPTH: u8;

    /// PNG color type.
    const COLOR_TYPE: ColorType;
}

impl Pixel for u8 {
    type Component = u8;
    const BIT_DEPTH: u8 = 8;
    const COLOR_TYPE: ColorType = ColorType::Index;
}

impl<T: PixelComponent> Pixel for Lum<T> {
    type Component = T;
    const BIT_DEPTH: u8 = T::BIT_DEPTH;
    const COLOR_TYPE: ColorType = ColorType::Gray;
}

impl<T: PixelComponent> Pixel for LumAlpha<T> {
    type Component = T;
    const BIT_DEPTH: u8 = T::BIT_DEPTH;
    const COLOR_TYPE: ColorType = ColorType::GrayAlpha;
}

impl<T: PixelComponent> Pixel for Rgb<T> {
    type Component = T;
    const BIT_DEPTH: u8 = T::BIT_DEPTH;
    const COLOR_TYPE: ColorType = ColorType::Rgb;
}

impl<T: PixelComponent> Pixel for Rgba<T> {
    type Component = T;
    const BIT_DEPTH: u8 = T::BIT_DEPTH;
    const COLOR_TYPE: ColorType = ColorType::RgbAlpha;
}

impl<'a, T: Pixel> Pixel for &'a T {
    type Component = T::Component;
    const BIT_DEPTH: u8 = T::BIT_DEPTH;
    const COLOR_TYPE: ColorType = T::COLOR_TYPE;
}

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

/// PNG encoder/decoder error type.
#[derive(Debug)]
pub enum Error {
    Arithmetic {
        source: Box<dyn 'static + Send + Sync + std::error::Error>,
    },
    BitDepth {
        bit_depth: u8,
        color_type: ColorType,
    },
    Crc,
    ChunkId {
        bytes: [u8; 4],
    },
    ChunkIdLen {
        len: usize,
    },
    ChunkLen {
        chunk_id: ChunkId,
        len: u32,
    },
    ColorType {
        raw: u8,
    },
    CompressionMethod {
        raw: u8,
    },
    CriticalChunk {
        chunk_id: ChunkId,
    },
    DuplicateChunk {
        chunk_id: ChunkId,
    },
    FilterByte {
        raw: u8,
    },
    FilterMethod {
        raw: u8,
    },
    ImageSize {
        size: Vector2<usize>,
    },
    InterlaceMethod {
        raw: u8,
    },
    InvalidArgument {
        detail: &'static str,
    },
    Io {
        source: std::io::Error,
    },
    MissingChunk {
        chunk_id: ChunkId,
    },
    MissingPalette,
    PaletteLen {
        len: usize,
    },
    Signature,
    UnexpectedChunk {
        chunk_id: ChunkId,
        detail: &'static str,
    },
    WrongChunk {
        expected: ChunkId,
        found: ChunkId,
    },
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match *self {
            Error::Arithmetic { ref source } => write!(fmt, "arithmetic error: {}", source),
            Error::BitDepth {
                bit_depth,
                color_type,
            } => write!(fmt, "invalid png bit depth: {} {}", color_type, bit_depth),
            Error::Crc => fmt.write_str("chunk crc mismatch"),
            Error::ChunkId { bytes } => {
                write!(
                    fmt,
                    "invalid png chunk id: {:02x} {:02x} {:02x} {:02x}",
                    bytes[0], bytes[1], bytes[2], bytes[3]
                )
            },
            Error::ChunkIdLen { len } => write!(fmt, "invalid png chunk id length: {}", len),
            Error::ChunkLen { chunk_id, len } => write!(
                fmt,
                "invalid/unexpected png chunk length: {}, {} bytes",
                chunk_id, len
            ),
            Error::ColorType { raw } => write!(fmt, "invalid png color type: {}", raw),
            Error::CompressionMethod { raw } => {
                write!(fmt, "invalid png compression method: {}", raw)
            },
            Error::CriticalChunk { chunk_id } => {
                write!(fmt, "unhandled critical png chunk: {}", chunk_id)
            },
            Error::DuplicateChunk { chunk_id } => write!(fmt, "duplicate png chunk: {}", chunk_id),
            Error::FilterByte { raw } => write!(fmt, "invalid png filter row byte: {}", raw),
            Error::FilterMethod { raw } => write!(fmt, "invalid png filter method: {}", raw),
            Error::ImageSize { size } => {
                write!(fmt, "invalid png image size: {}x{}", size.x, size.y)
            },
            Error::InterlaceMethod { raw } => write!(fmt, "invalid png interlace method: {}", raw),
            Error::InvalidArgument { detail } => write!(fmt, "invalid argument: {}", detail),
            Error::Io { ref source } => write!(fmt, "i/o error: {}", source),
            Error::MissingChunk { chunk_id } => write!(fmt, "missing png chunk: {}", chunk_id),
            Error::MissingPalette => fmt.write_str("missing palette"),
            Error::PaletteLen { len } => write!(fmt, "invalid png palette length: {}", len),
            Error::Signature => fmt.write_str("invalid png signature"),
            Error::UnexpectedChunk { chunk_id, detail } => {
                write!(fmt, "unexpected png chunk ({}): {}", chunk_id, detail)
            },
            Error::WrongChunk { expected, found } => write!(
                fmt,
                "wrong chunk id: expected {}, found {}",
                expected, found
            ),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(source: std::io::Error) -> Error {
        Error::Io { source }
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(source: std::num::TryFromIntError) -> Error {
        Error::Arithmetic {
            source: Box::new(source),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn 'static + std::error::Error)> {
        match *self {
            Error::Arithmetic { ref source } => Some(&**source),
            Error::Io { ref source } => Some(source),
            _ => None,
        }
    }
}
