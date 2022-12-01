/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use byteorder::{WriteBytesExt, BE};
use color::Rgb;
use math::{TryFromComposite, Vector2};

use crate::codec::png::compress::Compressor;
use crate::codec::png::filter::BaseFilterer;
use crate::codec::png::interlace::{Interlacer, InterlacerItem};
use crate::codec::png::pixel::PixelPacker;
use crate::codec::png::{
    ChunkId, ChunkWriter, ColorType, CompressionMethod, FilterMethod, Header, InterlaceMethod,
    InvalidBitDepth, Pixel,
};
use crate::image::Image;

const MAX_DIMENSION: u32 = 0x7fff_ffff;
const MAX_IDAT_SIZE: usize = 64 * 1024;
const MAX_PALETTE_LEN: usize = 256;

/// PNG image encoder.
pub struct Encoder<'i, 'p, I>
where
    I: 'i + Image,
    I::Pixel<'i>: 'i + Pixel,
{
    bit_depth: u8,
    compression_method: CompressionMethod,
    filter_method: FilterMethod,
    image: &'i I,
    interlace_method: Option<InterlaceMethod>,
    palette: Option<&'p [Rgb<u8>]>,
}

impl<'i, 'p, I> Encoder<'i, 'p, I>
where
    I: 'i + Image,
    I::Pixel<'i>: 'i + Pixel,
{
    /// Constructs an encoder for the specified image.
    pub fn new(image: &'i I) -> Self {
        Encoder {
            bit_depth: <I::Pixel<'i> as Pixel>::BIT_DEPTH,
            compression_method: CompressionMethod::Zlib,
            filter_method: FilterMethod::Base,
            image,
            interlace_method: None,
            palette: None,
        }
    }

    /// Changes the encoder's bit depth.
    pub fn with_bit_depth(&mut self, bit_depth: u8) -> Result<&mut Self, EncoderError> {
        <I::Pixel<'i> as Pixel>::COLOR_TYPE.check_bit_depth(bit_depth)?;
        self.bit_depth = bit_depth;
        Ok(self)
    }

    /// Changes the encoder's interlace method.
    pub fn with_interlace_method(
        &mut self, interlace_method: Option<InterlaceMethod>,
    ) -> &mut Self {
        self.interlace_method = interlace_method;
        self
    }

    /// Changes the encoder's palette. This is ignored if the color type is not indexed.
    pub fn with_palette(&mut self, palette: &'p [Rgb<u8>]) -> Result<&mut Self, EncoderError> {
        if palette.is_empty() || palette.len() > MAX_PALETTE_LEN {
            return Err(EncoderError::InvalidPaletteLen {
                palette_len: palette.len(),
            });
        }
        self.palette = Some(palette);
        Ok(self)
    }

    /// Encodes the image as a PNG stream.
    pub fn write<W: Write>(&self, w: &mut W) -> Result<(), EncoderError> {
        let header = Header::for_image(self.image);
        write_signature(w)?;
        write_IHDR(w, &header)?;

        if <I::Pixel<'i> as Pixel>::COLOR_TYPE == ColorType::Index {
            match self.palette {
                None => return Err(EncoderError::MissingPalette),
                Some(palette) => write_PLTE(w, palette)?,
            }
        }

        write_IDAT(
            w,
            self.image,
            self.bit_depth,
            self.compression_method,
            self.filter_method,
            self.interlace_method,
        )?;
        write_IEND(w)?;
        Ok(())
    }

    /// Encodes the image as a PNG file.
    pub fn write_file<P: AsRef<Path>>(&self, path: P) -> Result<(), EncoderError> {
        let mut writer = BufWriter::new(File::create(path)?);
        self.write(&mut writer)?;
        writer.flush()?;
        Ok(())
    }
}

/// PNG encoder error type.
#[derive(Debug)]
pub enum EncoderError {
    InvalidBitDepth { source: InvalidBitDepth },
    InvalidImageSize { image_size: Vector2<usize> },
    InvalidPaletteLen { palette_len: usize },
    IoError { source: std::io::Error },
    MissingPalette,
}

impl Display for EncoderError {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match *self {
            EncoderError::InvalidBitDepth { source } => {
                write!(fmt, "{}", source)
            },
            EncoderError::InvalidImageSize { image_size } => {
                write!(fmt, "invalid image size: {}x{}", image_size.x, image_size.y)
            },
            EncoderError::InvalidPaletteLen { palette_len } => {
                write!(fmt, "invalid palette length: {}", palette_len)
            },
            EncoderError::IoError { ref source } => {
                write!(fmt, "{}", source)
            },
            EncoderError::MissingPalette => fmt.write_str("missing palette"),
        }
    }
}

impl Error for EncoderError {
    fn source(&self) -> Option<&(dyn 'static + Error)> {
        match *self {
            EncoderError::InvalidBitDepth { ref source } => Some(source),
            EncoderError::IoError { ref source } => Some(source),
            _ => None,
        }
    }
}

impl From<std::io::Error> for EncoderError {
    fn from(source: std::io::Error) -> EncoderError {
        EncoderError::IoError { source }
    }
}

impl From<InvalidBitDepth> for EncoderError {
    fn from(source: InvalidBitDepth) -> EncoderError {
        EncoderError::InvalidBitDepth { source }
    }
}

/// Writes the PNG `IEND` chunk.
#[allow(non_snake_case)]
pub fn write_IEND<W: Write>(w: &mut W) -> Result<(), EncoderError> {
    ChunkWriter::new(w, ChunkId::IEND).finish()?;
    Ok(())
}

/// Writes the PNG `IDAT` pixel data chunk(s).
#[allow(non_snake_case)]
pub fn write_IDAT<'a, W, I>(
    w: &mut W, image: &'a I, bit_depth: u8, compression_method: CompressionMethod,
    _filter_method: FilterMethod, interlace_method: Option<InterlaceMethod>,
) -> Result<(), EncoderError>
where
    W: Write,
    I: Image,
    I::Pixel<'a>: Pixel,
{
    let chunk = ChunkWriter::new_progressive(w, ChunkId::IDAT, MAX_IDAT_SIZE);
    let mut compress = Compressor::new(chunk, compression_method);
    let mut filter = BaseFilterer::new(
        compress,
        image.size(),
        bit_depth,
        <I::Pixel<'a> as Pixel>::COLOR_TYPE,
    );
    let mut packer = PixelPacker::new(filter, bit_depth);

    for item in Interlacer::new(image.size(), interlace_method) {
        match item {
            InterlacerItem::BeginPass { size } => {
                compress = packer.finish()?.finish();
                filter = BaseFilterer::new(
                    compress,
                    size,
                    bit_depth,
                    <I::Pixel<'a> as Pixel>::COLOR_TYPE,
                );
                packer = PixelPacker::new(filter, bit_depth);
            },
            InterlacerItem::Pixel { pos } => {
                packer.pack(image.get_pixel(pos))?;
            },
        }
    }

    packer.finish()?.finish().finish()?.finish()?;
    Ok(())
}

/// Writes the PNG `IHDR` header chunk.
#[allow(non_snake_case)]
pub fn write_IHDR<W: Write>(w: &mut W, header: &Header) -> Result<(), EncoderError> {
    let size: Vector2<u32> = match TryFromComposite::try_from_composite(header.image_size) {
        Ok(size) => size,
        Err(_) => {
            return Err(EncoderError::InvalidImageSize {
                image_size: header.image_size,
            })
        },
    };
    if size.x == 0 || size.x > MAX_DIMENSION || size.y == 0 || size.y > MAX_DIMENSION {
        return Err(EncoderError::InvalidImageSize {
            image_size: header.image_size,
        });
    }
    header.color_type.check_bit_depth(header.bit_depth)?;

    let mut chunk = ChunkWriter::new(w, ChunkId::IHDR);
    chunk.write_u32::<BE>(size.x)?;
    chunk.write_u32::<BE>(size.y)?;
    chunk.write_u8(header.bit_depth)?;
    chunk.write_u8(header.color_type as u8)?;
    chunk.write_u8(header.compression_method as u8)?;
    chunk.write_u8(header.filter_method as u8)?;
    chunk.write_u8(InterlaceMethod::as_byte(header.interlace_method))?;
    chunk.finish()?;
    Ok(())
}

/// Writes the PNG `PLTE` palette chunk.
#[allow(non_snake_case)]
pub fn write_PLTE<W: Write>(w: &mut W, palette: &[Rgb<u8>]) -> Result<(), EncoderError> {
    if palette.is_empty() || palette.len() > MAX_PALETTE_LEN {
        return Err(EncoderError::InvalidPaletteLen {
            palette_len: palette.len(),
        });
    }

    let mut chunk = ChunkWriter::new(w, ChunkId::PLTE);
    for color in palette {
        let array: [u8; 3] = (*color).into();
        chunk.write_all(&array[..])?;
    }
    chunk.finish()?;
    Ok(())
}

/// Writes the PNG file signature.
pub fn write_signature<W: Write>(w: &mut W) -> Result<(), EncoderError> {
    w.write_all(&crate::codec::PNG_SIGNATURE[..])?;
    Ok(())
}
