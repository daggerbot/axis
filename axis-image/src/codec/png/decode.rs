/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::fs::File;
use std::io::Read;
use std::marker::PhantomData;
use std::path::Path;

use byteorder::{ByteOrder, ReadBytesExt, BE};
use color::{IntoComponentLossy, Lum, LumAlpha, Rgb, Rgba};
use math::{IntoLossy, Vector2};
use peekread::{BufPeekReader, PeekRead};

use crate::codec::png::chunk::{ChunkId, ChunkReader, ProgressiveChunkReader};
use crate::codec::png::compress::{CompressionMethod, Decompressor};
use crate::codec::png::filter::{Defilterer, FilterMethod};
use crate::codec::png::interlace::{InterlaceMethod, Interlacer, InterlacerItem};
use crate::codec::png::{self, ColorType, Error, Header, Pixel, PixelComponent};
use crate::image::ImageMut;
use crate::vec_image::VecImage;

/// Trait for decoding pixels from `u8`/`u16` sequences.
pub trait DecodePixel: Pixel {
    fn decode_pixel<E: Copy + IntoComponentLossy<Self::Component> + IntoLossy<Self::Component>>(
        buf: &[E],
    ) -> Self;
}

impl DecodePixel for u8 {
    fn decode_pixel<E: Copy + IntoComponentLossy<Self::Component> + IntoLossy<Self::Component>>(
        buf: &[E],
    ) -> Self {
        buf[0].into_lossy()
    }
}

impl<T: PixelComponent> DecodePixel for Lum<T> {
    fn decode_pixel<E: Copy + IntoComponentLossy<Self::Component> + IntoLossy<Self::Component>>(
        buf: &[E],
    ) -> Self {
        Lum {
            l: buf[0].into_component_lossy(),
        }
    }
}

impl<T: PixelComponent> DecodePixel for LumAlpha<T> {
    fn decode_pixel<E: Copy + IntoComponentLossy<Self::Component> + IntoLossy<Self::Component>>(
        buf: &[E],
    ) -> Self {
        LumAlpha {
            l: buf[0].into_component_lossy(),
            a: buf[1].into_component_lossy(),
        }
    }
}

impl<T: PixelComponent> DecodePixel for Rgb<T> {
    fn decode_pixel<E: Copy + IntoComponentLossy<Self::Component> + IntoLossy<Self::Component>>(
        buf: &[E],
    ) -> Self {
        Rgb {
            r: buf[0].into_component_lossy(),
            g: buf[1].into_component_lossy(),
            b: buf[2].into_component_lossy(),
        }
    }
}

impl<T: PixelComponent> DecodePixel for Rgba<T> {
    fn decode_pixel<E: Copy + IntoComponentLossy<Self::Component> + IntoLossy<Self::Component>>(
        buf: &[E],
    ) -> Self {
        Rgba {
            r: buf[0].into_component_lossy(),
            g: buf[1].into_component_lossy(),
            b: buf[2].into_component_lossy(),
            a: buf[3].into_component_lossy(),
        }
    }
}

/// Unpacks pixels, possibly with bit depths less than 8, from a byte stream.
struct PixelUnpacker<R: Read> {
    bit_depth: u8,
    byte: Option<u8>,
    inner: R,
    mask: u8,
    shift: u8,
}

impl<R: Read> PixelUnpacker<R> {
    fn into_inner(self) -> R {
        self.inner
    }

    fn new(inner: R, bit_depth: u8) -> PixelUnpacker<R> {
        PixelUnpacker {
            bit_depth,
            byte: None,
            inner,
            mask: ((1u32 << bit_depth) - 1) as u8,
            shift: 8 - bit_depth,
        }
    }

    /// Discards the rest of the current byte (for sub-byte bit depths).
    fn pad(&mut self) {
        self.byte = None;
        self.shift = 0;
    }

    fn unpack<P: DecodePixel>(&mut self) -> std::io::Result<P> {
        match self.bit_depth {
            1 | 2 | 4 => {
                let byte = match self.byte {
                    None => {
                        let byte = self.inner.read_u8()?;
                        self.byte = Some(byte);
                        byte
                    },
                    Some(byte) => byte,
                };
                let value = (byte >> self.shift) & self.mask;
                let pixel = P::decode_pixel(&[value]);
                match self.shift {
                    0 => {
                        self.byte = None;
                        self.shift = 8 - self.bit_depth;
                    },
                    _ => self.shift -= self.bit_depth,
                }
                Ok(pixel)
            },

            8 => {
                let mut buf = [0u8; 4];
                self.inner
                    .read_exact(&mut buf[..P::COLOR_TYPE.channel_count()])?;
                Ok(P::decode_pixel(&buf[..]))
            },

            16 => {
                let mut bytes = [0u8; 8];
                self.inner
                    .read_exact(&mut bytes[..(P::COLOR_TYPE.channel_count() * 2)])?;
                let words = [
                    <BE as ByteOrder>::read_u16(&bytes[0..]),
                    <BE as ByteOrder>::read_u16(&bytes[2..]),
                    <BE as ByteOrder>::read_u16(&bytes[4..]),
                    <BE as ByteOrder>::read_u16(&bytes[6..]),
                ];
                Ok(P::decode_pixel(&words[..]))
            },

            _ => unreachable!(),
        }
    }
}

/// Reads and decodes pixels from a PNG `IDAT` chunk.
pub struct PixelReader<R: PeekRead, P: DecodePixel> {
    bit_depth: u8,
    filter_method: FilterMethod,
    inner: Option<PixelUnpacker<Defilterer<Decompressor<ProgressiveChunkReader<R>>>>>,
    interlacer: Interlacer,
    _phantom: PhantomData<P>,
    prev_row: Option<usize>,
}

impl<R: PeekRead, P: DecodePixel> PixelReader<R, P> {
    pub fn finish(mut self) -> Result<R, Error> {
        self.inner
            .take()
            .unwrap()
            .into_inner()
            .into_inner()
            .into_inner()
            .finish()
    }

    pub fn new(
        inner: ProgressiveChunkReader<R>, header: &Header,
    ) -> Result<PixelReader<R, P>, Error> {
        if header.color_type != P::COLOR_TYPE {
            return Err(Error::InvalidArgument {
                detail: "png color type mismatch",
            });
        }

        header.color_type.check_bit_depth(header.bit_depth)?;

        Ok(PixelReader {
            bit_depth: header.bit_depth,
            filter_method: header.filter_method,
            inner: Some(PixelUnpacker::new(
                Defilterer::new(
                    header.filter_method,
                    Decompressor::new(inner, header.compression_method),
                    header.image_size,
                    header.bit_depth,
                    header.color_type,
                ),
                header.bit_depth,
            )),
            interlacer: Interlacer::new(header.image_size, header.interlace_method),
            _phantom: PhantomData,
            prev_row: None,
        })
    }

    /// Reads the next pixel from the stream. Returns the position and the pixel value. Note that
    /// due to interlacing, adjacent pixels may not be sequential.
    pub fn next_pixel(&mut self) -> Result<Option<(Vector2<usize>, P)>, Error> {
        loop {
            match self.interlacer.next() {
                None => return Ok(None),

                Some(InterlacerItem::BeginPass { size }) => {
                    let decompressor = self.inner.take().unwrap().into_inner().into_inner();
                    self.inner = Some(PixelUnpacker::new(
                        Defilterer::new(
                            self.filter_method,
                            decompressor,
                            size,
                            self.bit_depth,
                            P::COLOR_TYPE,
                        ),
                        self.bit_depth,
                    ));
                    self.prev_row = None;
                },

                Some(InterlacerItem::Pixel { pos }) => {
                    let unpacker = self.inner.as_mut().unwrap();
                    if self.prev_row != Some(pos.y) {
                        self.prev_row = Some(pos.y);
                        unpacker.pad();
                    }
                    return Ok(Some((pos, unpacker.unpack()?)));
                },
            }
        }
    }
}

/// Enumeration of PNG pixel readers.
pub enum AnyPixelReader<R: PeekRead> {
    Index(PixelReader<R, u8>),
    Gray8(PixelReader<R, Lum<u8>>),
    Gray16(PixelReader<R, Lum<u16>>),
    GrayAlpha8(PixelReader<R, LumAlpha<u8>>),
    GrayAlpha16(PixelReader<R, LumAlpha<u16>>),
    Rgb8(PixelReader<R, Rgb<u8>>),
    Rgb16(PixelReader<R, Rgb<u16>>),
    RgbAlpha8(PixelReader<R, Rgba<u8>>),
    RgbAlpha16(PixelReader<R, Rgba<u16>>),
}

/// Fully read and decoded PNG image buffer.
pub enum DecodedImage {
    Index {
        image: VecImage<u8>,
        palette: Vec<Rgb<u8>>,
    },
    Gray8 {
        image: VecImage<Lum<u8>>,
    },
    Gray16 {
        image: VecImage<Lum<u16>>,
    },
    GrayAlpha8 {
        image: VecImage<LumAlpha<u8>>,
    },
    GrayAlpha16 {
        image: VecImage<LumAlpha<u16>>,
    },
    Rgb8 {
        image: VecImage<Rgb<u8>>,
    },
    Rgb16 {
        image: VecImage<Rgb<u16>>,
    },
    RgbAlpha8 {
        image: VecImage<Rgba<u8>>,
    },
    RgbAlpha16 {
        image: VecImage<Rgba<u16>>,
    },
}

/// Reads a PNG stream.
pub fn read<R: Read>(r: &mut R) -> Result<DecodedImage, Error> {
    let mut r = BufPeekReader::new(r);
    read_signature(&mut r)?;

    let mut header = None;
    let mut palette = None;
    let mut image = None;

    loop {
        let chunk = ChunkReader::new(&mut r)?;
        let chunk_id = chunk.chunk_id();

        match chunk_id {
            ChunkId::IEND => match image {
                None => {
                    return Err(Error::MissingChunk {
                        chunk_id: ChunkId::IDAT,
                    })
                },
                Some(image) => return Ok(image),
            },

            ChunkId::IDAT => match image {
                None => {
                    let header = match header {
                        None => {
                            return Err(Error::WrongChunk {
                                expected: ChunkId::IHDR,
                                found: chunk_id,
                            })
                        },
                        Some(ref header) => header,
                    };

                    match read_IDAT(header, ProgressiveChunkReader::new(chunk))? {
                        AnyPixelReader::Index(mut pixel_reader) => match palette.take() {
                            None => {
                                return Err(Error::WrongChunk {
                                    expected: ChunkId::PLTE,
                                    found: chunk_id,
                                })
                            },
                            Some(palette) => {
                                let mut buf = VecImage::new(header.image_size);
                                while let Some((pos, pixel)) = pixel_reader.next_pixel()? {
                                    buf.set_pixel(pos, pixel);
                                }
                                pixel_reader.finish()?;
                                image = Some(DecodedImage::Index {
                                    image: buf,
                                    palette,
                                });
                            },
                        },

                        AnyPixelReader::Gray8(mut pixel_reader) => {
                            let mut buf = VecImage::new(header.image_size);
                            while let Some((pos, pixel)) = pixel_reader.next_pixel()? {
                                buf.set_pixel(pos, pixel);
                            }
                            pixel_reader.finish()?;
                            image = Some(DecodedImage::Gray8 { image: buf });
                        },

                        AnyPixelReader::Gray16(mut pixel_reader) => {
                            let mut buf = VecImage::new(header.image_size);
                            while let Some((pos, pixel)) = pixel_reader.next_pixel()? {
                                buf.set_pixel(pos, pixel);
                            }
                            pixel_reader.finish()?;
                            image = Some(DecodedImage::Gray16 { image: buf });
                        },

                        AnyPixelReader::GrayAlpha8(mut pixel_reader) => {
                            let mut buf = VecImage::new(header.image_size);
                            while let Some((pos, pixel)) = pixel_reader.next_pixel()? {
                                buf.set_pixel(pos, pixel);
                            }
                            pixel_reader.finish()?;
                            image = Some(DecodedImage::GrayAlpha8 { image: buf });
                        },

                        AnyPixelReader::GrayAlpha16(mut pixel_reader) => {
                            let mut buf = VecImage::new(header.image_size);
                            while let Some((pos, pixel)) = pixel_reader.next_pixel()? {
                                buf.set_pixel(pos, pixel);
                            }
                            pixel_reader.finish()?;
                            image = Some(DecodedImage::GrayAlpha16 { image: buf });
                        },

                        AnyPixelReader::Rgb8(mut pixel_reader) => {
                            let mut buf = VecImage::new(header.image_size);
                            while let Some((pos, pixel)) = pixel_reader.next_pixel()? {
                                buf.set_pixel(pos, pixel);
                            }
                            pixel_reader.finish()?;
                            image = Some(DecodedImage::Rgb8 { image: buf });
                        },

                        AnyPixelReader::Rgb16(mut pixel_reader) => {
                            let mut buf = VecImage::new(header.image_size);
                            while let Some((pos, pixel)) = pixel_reader.next_pixel()? {
                                buf.set_pixel(pos, pixel);
                            }
                            pixel_reader.finish()?;
                            image = Some(DecodedImage::Rgb16 { image: buf });
                        },

                        AnyPixelReader::RgbAlpha8(mut pixel_reader) => {
                            let mut buf = VecImage::new(header.image_size);
                            while let Some((pos, pixel)) = pixel_reader.next_pixel()? {
                                buf.set_pixel(pos, pixel);
                            }
                            pixel_reader.finish()?;
                            image = Some(DecodedImage::RgbAlpha8 { image: buf });
                        },

                        AnyPixelReader::RgbAlpha16(mut pixel_reader) => {
                            let mut buf = VecImage::new(header.image_size);
                            while let Some((pos, pixel)) = pixel_reader.next_pixel()? {
                                buf.set_pixel(pos, pixel);
                            }
                            pixel_reader.finish()?;
                            image = Some(DecodedImage::RgbAlpha16 { image: buf });
                        },
                    }
                },

                Some(_) => return Err(Error::DuplicateChunk { chunk_id }),
            },

            ChunkId::IHDR => match header {
                None => header = Some(read_IHDR(chunk)?),
                Some(_) => return Err(Error::DuplicateChunk { chunk_id }),
            },

            ChunkId::PLTE => match palette {
                None => match header {
                    None => {
                        return Err(Error::WrongChunk {
                            expected: ChunkId::IHDR,
                            found: chunk_id,
                        })
                    },
                    Some(ref header) => match header.color_type {
                        ColorType::Index => palette = Some(read_PLTE(chunk)?),
                        _ => {
                            return Err(Error::UnexpectedChunk {
                                chunk_id,
                                detail: "not an indexed image",
                            })
                        },
                    },
                },
                Some(_) => return Err(Error::DuplicateChunk { chunk_id }),
            },

            _ => {
                if chunk_id.is_critical() {
                    return Err(Error::CriticalChunk { chunk_id });
                }
            },
        }
    }
}

/// Reads and decodes a PNG stream from a file.
pub fn read_file<P: AsRef<Path>>(path: P) -> Result<DecodedImage, Error> {
    read(&mut File::open(path)?)
}

/// Reads the contents of a series of PNG `IDAT` chunks.
#[allow(non_snake_case)]
pub fn read_IDAT<R: PeekRead>(
    header: &Header, chunk: ProgressiveChunkReader<R>,
) -> Result<AnyPixelReader<R>, Error> {
    let chunk_id = chunk.chunk_id();
    match chunk_id {
        ChunkId::IDAT => (),
        found => {
            return Err(Error::WrongChunk {
                expected: ChunkId::IHDR,
                found,
            });
        },
    }

    // Make sure the header is at least somewhat valid.
    header.color_type.check_bit_depth(header.bit_depth)?;

    match (header.color_type, header.bit_depth) {
        (ColorType::Index, 1..=8) => Ok(AnyPixelReader::Index(PixelReader::new(chunk, header)?)),
        (ColorType::Gray, 1..=8) => Ok(AnyPixelReader::Gray8(PixelReader::new(chunk, header)?)),
        (ColorType::Gray, 16) => Ok(AnyPixelReader::Gray16(PixelReader::new(chunk, header)?)),
        (ColorType::GrayAlpha, 8) => {
            Ok(AnyPixelReader::GrayAlpha8(PixelReader::new(chunk, header)?))
        },
        (ColorType::GrayAlpha, 16) => Ok(AnyPixelReader::GrayAlpha16(PixelReader::new(
            chunk, header,
        )?)),
        (ColorType::Rgb, 8) => Ok(AnyPixelReader::Rgb8(PixelReader::new(chunk, header)?)),
        (ColorType::Rgb, 16) => Ok(AnyPixelReader::Rgb16(PixelReader::new(chunk, header)?)),
        (ColorType::RgbAlpha, 8) => Ok(AnyPixelReader::RgbAlpha8(PixelReader::new(chunk, header)?)),
        (ColorType::RgbAlpha, 16) => {
            Ok(AnyPixelReader::RgbAlpha16(PixelReader::new(chunk, header)?))
        },
        _ => unreachable!(),
    }
}

/// Reads the contents of a PNG `IHDR` chunk.
#[allow(non_snake_case)]
pub fn read_IHDR<R: Read>(mut chunk: ChunkReader<R>) -> Result<Header, Error> {
    let chunk_id = chunk.chunk_id();
    match chunk_id {
        ChunkId::IHDR => (),
        found => {
            return Err(Error::WrongChunk {
                expected: ChunkId::IHDR,
                found,
            });
        },
    }
    match chunk.chunk_len() {
        png::IHDR_LENGTH => (),
        len => return Err(Error::ChunkLen { chunk_id, len }),
    }

    let width = chunk.read_u32::<BE>()?;
    let height = chunk.read_u32::<BE>()?;
    let size = Vector2::new(usize::try_from(width)?, usize::try_from(height)?);
    if width == 0 || width > png::MAX_DIMENSION || height == 0 || height > png::MAX_DIMENSION {
        return Err(Error::ImageSize { size });
    }
    let bit_depth = chunk.read_u8()?;
    let color_type = ColorType::try_from(chunk.read_u8()?)?;
    color_type.check_bit_depth(bit_depth)?;
    let compression_method = CompressionMethod::try_from(chunk.read_u8()?)?;
    let filter_method = FilterMethod::try_from(chunk.read_u8()?)?;
    let interlace_method = InterlaceMethod::from_byte(chunk.read_u8()?)?;

    chunk.finish()?;

    Ok(Header {
        bit_depth,
        color_type,
        compression_method,
        filter_method,
        image_size: size,
        interlace_method,
    })
}

/// Reads the contents of a PNG `PLTE` chunk.
#[allow(non_snake_case)]
pub fn read_PLTE<R: Read>(mut chunk: ChunkReader<R>) -> Result<Vec<Rgb<u8>>, Error> {
    let chunk_id = chunk.chunk_id();
    match chunk_id {
        ChunkId::PLTE => (),
        found => {
            return Err(Error::WrongChunk {
                expected: ChunkId::PLTE,
                found,
            });
        },
    }
    let chunk_len = chunk.chunk_len();
    if chunk_len == 0 || chunk_len > png::MAX_PALETTE_LEN as u32 * 3 || chunk_len % 3 != 0 {
        return Err(Error::ChunkLen {
            chunk_id,
            len: chunk_len,
        });
    }

    let mut palette = Vec::new();
    let mut bytes = [0; 3];
    let len = (chunk_len / 3) as usize;
    for _ in 0..len {
        chunk.read_exact(&mut bytes[..])?;
        palette.push(Rgb::new(bytes[0], bytes[1], bytes[2]));
    }

    chunk.finish()?;

    Ok(palette)
}

/// Reads and checks the PNG file signature.
pub fn read_signature<R: Read>(r: &mut R) -> Result<(), Error> {
    let mut signature = [0; crate::codec::PNG_SIGNATURE.len()];
    r.read_exact(&mut signature[..])?;
    if signature != crate::codec::PNG_SIGNATURE {
        return Err(Error::Signature);
    }
    Ok(())
}
