/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io::Write;

use byteorder::{ByteOrder, WriteBytesExt, BE};
use color::{FromComponentLossy, IntoComponentLossy, Lum, LumAlpha, Rgb, Rgba};

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
    pub fn check_bit_depth(self, bit_depth: u8) -> Result<u8, InvalidBitDepth> {
        match self {
            ColorType::Gray => match bit_depth {
                1 | 2 | 4 | 8 | 16 => Ok(bit_depth),
                _ => Err(InvalidBitDepth {
                    bit_depth,
                    color_type: Some(self),
                }),
            },
            ColorType::Rgb => match bit_depth {
                8 | 16 => Ok(bit_depth),
                _ => Err(InvalidBitDepth {
                    bit_depth,
                    color_type: Some(self),
                }),
            },
            ColorType::Index => match bit_depth {
                1 | 2 | 4 | 8 => Ok(bit_depth),
                _ => Err(InvalidBitDepth {
                    bit_depth,
                    color_type: Some(self),
                }),
            },
            ColorType::GrayAlpha => match bit_depth {
                8 | 16 => Ok(bit_depth),
                _ => Err(InvalidBitDepth {
                    bit_depth,
                    color_type: Some(self),
                }),
            },
            ColorType::RgbAlpha => match bit_depth {
                8 | 16 => Ok(bit_depth),
                _ => Err(InvalidBitDepth {
                    bit_depth,
                    color_type: Some(self),
                }),
            },
        }
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
    type Error = InvalidColorType;

    fn try_from(byte: u8) -> Result<ColorType, InvalidColorType> {
        match byte {
            0 => Ok(ColorType::Gray),
            2 => Ok(ColorType::Rgb),
            3 => Ok(ColorType::Index),
            4 => Ok(ColorType::GrayAlpha),
            6 => Ok(ColorType::RgbAlpha),
            _ => Err(InvalidColorType(byte)),
        }
    }
}

/// Raised when an invalid PNG color type is encountered.
#[derive(Clone, Copy, Debug)]
pub struct InvalidColorType(pub u8);

impl InvalidColorType {
    const DESCRIPTION: &'static str = "invalid png color type";
}

impl Display for InvalidColorType {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}: {}", Self::DESCRIPTION, self.0)
    }
}

impl Error for InvalidColorType {
    fn description(&self) -> &str {
        Self::DESCRIPTION
    }
}

/// Raised when an invalid bit depth is encountered.
#[derive(Clone, Copy, Debug)]
pub struct InvalidBitDepth {
    pub bit_depth: u8,
    pub color_type: Option<ColorType>,
}

impl InvalidBitDepth {
    const DESCRIPTION: &'static str = "invalid png bit depth";
}

impl Display for InvalidBitDepth {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match self.color_type {
            None => {
                write!(fmt, "{}: {}", Self::DESCRIPTION, self.bit_depth)
            },
            Some(color_type) => {
                write!(
                    fmt,
                    "{}: {} {}",
                    Self::DESCRIPTION,
                    color_type,
                    self.bit_depth
                )
            },
        }
    }
}

impl Error for InvalidBitDepth {
    fn description(&self) -> &str {
        Self::DESCRIPTION
    }
}

/// Trait for PNG pixel components.
pub trait PixelComponent:
    Copy
    + FromComponentLossy<u8>
    + FromComponentLossy<u16>
    + IntoComponentLossy<u8>
    + IntoComponentLossy<u16>
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
pub trait Pixel: Sized {
    /// Pixel component type.
    type Component;

    /// Default bit depth.
    const BIT_DEPTH: u8;

    /// PNG color type.
    const COLOR_TYPE: ColorType;

    /// Encodes the pixel as an array of `u8`.
    fn encode_u8(&self, buf: &mut [u8]);

    /// Encodes the pixel as an array of `u16`.
    fn encode_u16(&self, buf: &mut [u16]);
}

impl Pixel for u8 {
    type Component = u8;

    const BIT_DEPTH: u8 = 8;
    const COLOR_TYPE: ColorType = ColorType::Index;

    fn encode_u8(&self, buf: &mut [u8]) {
        buf[0] = *self;
    }

    fn encode_u16(&self, buf: &mut [u16]) {
        buf[0] = *self as u16;
    }
}

impl<T: PixelComponent> Pixel for Lum<T> {
    type Component = T;

    const BIT_DEPTH: u8 = T::BIT_DEPTH;
    const COLOR_TYPE: ColorType = ColorType::Gray;

    fn encode_u8(&self, buf: &mut [u8]) {
        buf[0] = self.l.into_component_lossy();
    }

    fn encode_u16(&self, buf: &mut [u16]) {
        buf[0] = self.l.into_component_lossy();
    }
}

impl<T: PixelComponent> Pixel for LumAlpha<T> {
    type Component = T;

    const BIT_DEPTH: u8 = T::BIT_DEPTH;
    const COLOR_TYPE: ColorType = ColorType::GrayAlpha;

    fn encode_u8(&self, buf: &mut [u8]) {
        buf[0] = self.l.into_component_lossy();
        buf[1] = self.a.into_component_lossy();
    }

    fn encode_u16(&self, buf: &mut [u16]) {
        buf[0] = self.l.into_component_lossy();
        buf[1] = self.a.into_component_lossy();
    }
}

impl<T: PixelComponent> Pixel for Rgb<T> {
    type Component = T;

    const BIT_DEPTH: u8 = T::BIT_DEPTH;
    const COLOR_TYPE: ColorType = ColorType::Rgb;

    fn encode_u8(&self, buf: &mut [u8]) {
        buf[0] = self.r.into_component_lossy();
        buf[1] = self.g.into_component_lossy();
        buf[2] = self.b.into_component_lossy();
    }

    fn encode_u16(&self, buf: &mut [u16]) {
        buf[0] = self.r.into_component_lossy();
        buf[1] = self.g.into_component_lossy();
        buf[2] = self.b.into_component_lossy();
    }
}

impl<T: PixelComponent> Pixel for Rgba<T> {
    type Component = T;

    const BIT_DEPTH: u8 = T::BIT_DEPTH;
    const COLOR_TYPE: ColorType = ColorType::RgbAlpha;

    fn encode_u8(&self, buf: &mut [u8]) {
        buf[0] = self.r.into_component_lossy();
        buf[1] = self.g.into_component_lossy();
        buf[2] = self.b.into_component_lossy();
        buf[3] = self.a.into_component_lossy();
    }

    fn encode_u16(&self, buf: &mut [u16]) {
        buf[0] = self.r.into_component_lossy();
        buf[1] = self.g.into_component_lossy();
        buf[2] = self.b.into_component_lossy();
        buf[3] = self.a.into_component_lossy();
    }
}

impl<'a, T: Pixel> Pixel for &'a T {
    type Component = T::Component;

    const BIT_DEPTH: u8 = T::BIT_DEPTH;
    const COLOR_TYPE: ColorType = T::COLOR_TYPE;

    fn encode_u8(&self, buf: &mut [u8]) {
        (*self).encode_u8(buf);
    }

    fn encode_u16(&self, buf: &mut [u16]) {
        (*self).encode_u16(buf);
    }
}

/// Packs pixels into bytes.
pub struct PixelPacker<W: Write> {
    bit_depth: u8,
    byte: u8,
    inner: W,
    mask: u8,
    pos: u8,
}

impl<W: Write> PixelPacker<W> {
    pub fn finish(mut self) -> std::io::Result<W> {
        self.pad()?;
        Ok(self.inner)
    }

    pub fn new(inner: W, bit_depth: u8) -> PixelPacker<W> {
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

    pub fn pack<P: Pixel>(&mut self, pixel: P) -> std::io::Result<()> {
        match self.bit_depth {
            1 | 2 | 4 => {
                let mut bytes = [0; 1];
                pixel.encode_u8(&mut bytes);
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
                let mut bytes = [0; 4];
                pixel.encode_u8(&mut bytes);
                self.inner
                    .write_all(&bytes[..P::COLOR_TYPE.channel_count()])
            },
            16 => {
                let mut words = [0; 4];
                let mut bytes = [0; 8];
                pixel.encode_u16(&mut words);
                for i in 0..P::COLOR_TYPE.channel_count() {
                    BE::write_u16(&mut bytes[(i * 2)..], words[i]);
                }
                self.inner
                    .write_all(&bytes[..(P::COLOR_TYPE.channel_count() * 2)])
            },
            _ => unreachable!(),
        }
    }

    pub fn pad(&mut self) -> std::io::Result<()> {
        if self.pos != 0 {
            let byte = self.byte;
            self.byte = 0;
            self.pos = 0;
            self.inner.write_u8(byte)?;
        }
        Ok(())
    }
}
