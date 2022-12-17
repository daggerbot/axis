/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use byteorder::{ByteOrder, WriteBytesExt, BE};
use color::{IntoComponentLossy, Lum, LumAlpha, Rgb, Rgba};
use math::{IntoLossy, TryFromComposite, Vector2};

use crate::codec::png::chunk::{ChunkId, ChunkWriter};
use crate::codec::png::compress::{CompressionMethod, Compressor};
use crate::codec::png::filter::{FilterMethod, Filterer};
use crate::codec::png::interlace::{InterlaceMethod, Interlacer, InterlacerItem};
use crate::codec::png::{self, ColorType, Error, Header, Pixel, PixelComponent};
use crate::image::Image;

const MAX_IDAT_SIZE: usize = 64 * 1024;

/// Encodes an image as a PNG stream.
pub struct Encoder<'i, 'p, I>
where
    I: 'i + Image,
    I::Pixel<'i>: 'i + EncodePixel,
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
    I::Pixel<'i>: 'i + EncodePixel,
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
    pub fn with_bit_depth(&mut self, bit_depth: u8) -> Result<&mut Self, Error> {
        <I::Pixel<'i> as Pixel>::COLOR_TYPE.check_bit_depth(bit_depth)?;
        self.bit_depth = bit_depth;
        Ok(self)
    }

    /// Changes the encoder's interlace method.
    pub fn with_interlace_method(&mut self, interlace_method: Option<InterlaceMethod>)
        -> &mut Self
    {
        self.interlace_method = interlace_method;
        self
    }

    /// Changes the encoder's palette. This is ignored if the color type is not indexed.
    pub fn with_palette(&mut self, palette: &'p [Rgb<u8>]) -> Result<&mut Self, Error> {
        if palette.is_empty() || palette.len() > png::MAX_PALETTE_LEN {
            return Err(Error::PaletteLen { len: palette.len() });
        }
        self.palette = Some(palette);
        Ok(self)
    }

    /// Writes the encoded image.
    pub fn write<W: Write>(&self, w: &mut W) -> Result<(), Error> {
        let header = Header::for_image(self.image);
        write_signature(w)?;
        write_IHDR(w, &header)?;

        if <I::Pixel<'i> as Pixel>::COLOR_TYPE == ColorType::Index {
            match self.palette {
                None => return Err(Error::MissingPalette),
                Some(palette) => write_PLTE(w, palette)?,
            }
        }

        write_IDAT(w, self.image, self.bit_depth, self.compression_method, self.filter_method,
                   self.interlace_method)?;
        write_IEND(w)?;
        Ok(())
    }

    /// Encodes the image as a PNG file.
    pub fn write_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        let mut writer = BufWriter::new(File::create(path)?);
        self.write(&mut writer)?;
        writer.flush()?;
        Ok(())
    }
}

/// Trait for encoding pixels as `u8`/`u16` sequences.
pub trait EncodePixel: Pixel {
    fn encode_pixel<E>(self, buf: &mut [E])
    where
        Self::Component: IntoComponentLossy<E> + IntoLossy<E>;
}

impl EncodePixel for u8 {
    fn encode_pixel<E>(self, buf: &mut [E])
    where
        Self::Component: IntoComponentLossy<E> + IntoLossy<E>,
    {
        buf[0] = self.into_lossy();
    }
}

impl<T: PixelComponent> EncodePixel for Lum<T> {
    fn encode_pixel<E>(self, buf: &mut [E])
    where
        Self::Component: IntoComponentLossy<E> + IntoLossy<E>,
    {
        buf[0] = self.l.into_component_lossy();
    }
}

impl<T: PixelComponent> EncodePixel for LumAlpha<T> {
    fn encode_pixel<E>(self, buf: &mut [E])
    where
        Self::Component: IntoComponentLossy<E> + IntoLossy<E>,
    {
        buf[0] = self.l.into_component_lossy();
        buf[1] = self.a.into_component_lossy();
    }
}

impl<T: PixelComponent> EncodePixel for Rgb<T> {
    fn encode_pixel<E>(self, buf: &mut [E])
    where
        Self::Component: IntoComponentLossy<E> + IntoLossy<E>,
    {
        buf[0] = self.r.into_component_lossy();
        buf[1] = self.g.into_component_lossy();
        buf[2] = self.b.into_component_lossy();
    }
}

impl<T: PixelComponent> EncodePixel for Rgba<T> {
    fn encode_pixel<E>(self, buf: &mut [E])
    where
        Self::Component: IntoComponentLossy<E> + IntoLossy<E>,
    {
        buf[0] = self.r.into_component_lossy();
        buf[1] = self.g.into_component_lossy();
        buf[2] = self.b.into_component_lossy();
        buf[3] = self.a.into_component_lossy();
    }
}

impl<'a, T: EncodePixel> EncodePixel for &'a T {
    fn encode_pixel<E>(self, buf: &mut [E])
    where
        Self::Component: IntoComponentLossy<E> + IntoLossy<E>,
    {
        (*self).encode_pixel(buf)
    }
}

/// Writes packed pixels to the inner writer.
struct PixelPacker<W: Write> {
    bit_depth: u8,
    byte: u8,
    inner: W,
    mask: u8,
    pos: u8,
}

impl<W: Write> PixelPacker<W> {
    /// Writes the current byte (if a partial byte has been packed) and returns the inner writer.
    fn finish(mut self) -> std::io::Result<W> {
        self.pad()?;
        Ok(self.inner)
    }

    /// Constructs a pixel packer.
    fn new(inner: W, bit_depth: u8) -> PixelPacker<W> {
        PixelPacker {
            bit_depth: match bit_depth {
                1 | 2 | 4 | 8 | 16 => bit_depth,
                _ => unreachable!(),
            },
            byte: 0,
            inner,
            mask: ((1u32 << bit_depth) - 1) as u8,
            pos: 0,
        }
    }

    /// Packs a pixel into the data stream.
    fn pack<P: EncodePixel>(&mut self, pixel: P) -> std::io::Result<()> {
        match self.bit_depth {
            1 | 2 | 4 => {
                let mut bytes = [0u8; 1];
                pixel.encode_pixel(&mut bytes);
                self.byte |= (bytes[0] & self.mask) << (8 - self.bit_depth - self.pos);
                self.pos += self.bit_depth;
                if self.pos == 8 {
                    let byte = self.byte;
                    self.byte = 0;
                    self.pos = 0;
                    self.inner.write_u8(byte)?;
                }
                Ok(())
            },

            8 => {
                let mut bytes = [0u8; 4];
                pixel.encode_pixel(&mut bytes);
                self.inner
                    .write_all(&bytes[..P::COLOR_TYPE.channel_count()])
            },

            16 => {
                let mut words = [0u16; 4];
                let mut bytes = [0u8; 8];
                pixel.encode_pixel(&mut words);
                for i in 0..P::COLOR_TYPE.channel_count() {
                    <BE as ByteOrder>::write_u16(&mut bytes[(i * 2)..], words[i]);
                }
                self.inner
                    .write_all(&bytes[..(P::COLOR_TYPE.channel_count() * 2)])
            },

            _ => unreachable!(),
        }
    }

    /// Fills the rest of the current byte with zero if any pixel components have been packed into
    /// it.
    fn pad(&mut self) -> std::io::Result<()> {
        if self.pos != 0 {
            let byte = self.byte;
            self.byte = 0;
            self.pos = 0;
            self.inner.write_u8(byte)?;
        }
        Ok(())
    }
}

/// Writes the PNG `IEND` chunk.
#[allow(non_snake_case)]
pub fn write_IEND<W: Write>(w: &mut W) -> Result<(), Error> {
    ChunkWriter::new(w, ChunkId::IEND).finish()?;
    Ok(())
}

/// Writes the PNG pixel data stream consisting of one or more `IDAT` chunks.
#[allow(non_snake_case)]
pub fn write_IDAT<'a, W, I>(w: &mut W, image: &'a I, bit_depth: u8,
                            compression_method: CompressionMethod, filter_method: FilterMethod,
                            interlace_method: Option<InterlaceMethod>)
                            -> Result<(), Error>
where
    W: Write,
    I: Image,
    I::Pixel<'a>: EncodePixel,
{
    let chunk = ChunkWriter::new_progressive(w, ChunkId::IDAT, MAX_IDAT_SIZE);
    let mut compress = Compressor::new(chunk, compression_method);
    let mut filter = Filterer::new(
        filter_method,
        compress,
        image.size(),
        bit_depth,
        <I::Pixel<'a> as Pixel>::COLOR_TYPE,
    );
    let mut packer = PixelPacker::new(filter, bit_depth);

    for item in Interlacer::new(image.size(), interlace_method) {
        match item {
            InterlacerItem::BeginPass { size } => {
                compress = packer.finish()?.into_inner();
                filter = Filterer::new(
                    filter_method,
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

    packer.finish()?.into_inner().finish()?.finish()?;
    Ok(())
}

/// Writes the PNG `IHDR` header chunk.
#[allow(non_snake_case)]
pub fn write_IHDR<W: Write>(w: &mut W, header: &Header) -> Result<(), Error> {
    let size: Vector2<u32> = match TryFromComposite::try_from_composite(header.image_size) {
        Ok(size) => size,
        Err(_) => {
            return Err(Error::ImageSize {
                size: header.image_size,
            })
        },
    };
    if size.x == 0 || size.x > png::MAX_DIMENSION || size.y == 0 || size.y > png::MAX_DIMENSION {
        return Err(Error::ImageSize {
            size: header.image_size,
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
pub fn write_PLTE<W: Write>(w: &mut W, palette: &[Rgb<u8>]) -> Result<(), Error> {
    if palette.is_empty() || palette.len() > png::MAX_PALETTE_LEN {
        return Err(Error::PaletteLen { len: palette.len() });
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
pub fn write_signature<W: Write>(w: &mut W) -> Result<(), Error> {
    w.write_all(&crate::codec::PNG_SIGNATURE[..])?;
    Ok(())
}
