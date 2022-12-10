/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::fmt::{Display, Formatter};
use std::io::{Read, Write};

use byteorder::{ReadBytesExt, WriteBytesExt};
use math::{DivCeil, Vector2};

use crate::codec::png::{ColorType, Error};

/// Enumeration of PNG filter methods.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum FilterMethod {
    Base = 0,
}

impl FilterMethod {
    const fn description(self) -> &'static str {
        match self {
            FilterMethod::Base => "base",
        }
    }
}

impl Display for FilterMethod {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fmt.write_str(self.description())
    }
}

impl TryFrom<u8> for FilterMethod {
    type Error = Error;

    fn try_from(raw: u8) -> Result<FilterMethod, Error> {
        match raw {
            0 => Ok(FilterMethod::Base),
            _ => Err(Error::FilterMethod { raw }),
        }
    }
}

/// Filters bytes for better compression.
pub struct BaseFilterer<W: Write> {
    bytes_per_row: usize,
    height: usize,
    inner: W,
    row_byte_index: usize,
    row_index: usize,
    row_prefix_written: bool,
}

impl<W: Write> BaseFilterer<W> {
    pub fn into_inner(self) -> W {
        self.inner
    }

    pub fn new(
        inner: W, image_size: Vector2<usize>, bit_depth: u8, color_type: ColorType,
    ) -> BaseFilterer<W> {
        let bytes_per_row = match bit_depth {
            1 | 2 | 4 => DivCeil::div_ceil(image_size.x, 8 / bit_depth as usize),
            8 => image_size.x * color_type.channel_count(),
            16 => image_size.x * color_type.channel_count() * 2,
            _ => unreachable!(),
        };

        BaseFilterer {
            bytes_per_row,
            height: image_size.y,
            inner,
            row_byte_index: 0,
            row_index: 0,
            row_prefix_written: false,
        }
    }
}

impl<W: Write> Write for BaseFilterer<W> {
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.bytes_per_row == 0 || self.row_index == self.height || buf.is_empty() {
            return Ok(0);
        }

        // Write the row prefix. Right now we're always just writing 0, which bypasses filtering.
        // This makes for a valid PNG stream but does not make for good compression. See github
        // issue #1.
        if !self.row_prefix_written {
            self.inner.write_u8(0)?;
            self.row_prefix_written = true;
        }

        // Write data to the inner buffer.
        let n_to_write = std::cmp::min(buf.len(), self.bytes_per_row - self.row_byte_index);
        let n_written = self.inner.write(&buf[..n_to_write])?;
        if n_written == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
        }
        assert!(n_written <= n_to_write);
        self.row_byte_index += n_written;

        // Advance to the next row.
        if self.row_byte_index == self.bytes_per_row {
            self.row_byte_index = 0;
            self.row_index += 1;
            self.row_prefix_written = false;
        }

        Ok(n_written)
    }
}

/// Filters bytes for better compression. Sum of all supported filter methods.
pub enum Filterer<W: Write> {
    Base(BaseFilterer<W>),
}

impl<W: Write> Filterer<W> {
    pub fn into_inner(self) -> W {
        match self {
            Filterer::Base(f) => f.into_inner(),
        }
    }

    pub fn new(
        filter_method: FilterMethod, inner: W, image_size: Vector2<usize>, bit_depth: u8,
        color_type: ColorType,
    ) -> Filterer<W> {
        match filter_method {
            FilterMethod::Base => {
                Filterer::Base(BaseFilterer::new(inner, image_size, bit_depth, color_type))
            },
        }
    }
}

impl<W: Write> Write for Filterer<W> {
    fn flush(&mut self) -> std::io::Result<()> {
        match *self {
            Filterer::Base(ref mut f) => f.flush(),
        }
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match *self {
            Filterer::Base(ref mut f) => f.write(buf),
        }
    }
}

/// Reverses pixel filtering when decoding a PNG stream.
pub struct BaseDefilterer<R: Read> {
    bytes_per_row: usize,
    height: usize,
    inner: R,
    prev_row_data: Vec<u8>,
    row_byte_index: usize,
    row_data: Vec<u8>,
    row_index: usize,
    row_prefix: Option<u8>,
    sub_pitch: usize,
}

impl<R: Read> BaseDefilterer<R> {
    pub fn into_inner(self) -> R {
        self.inner
    }

    pub fn new(
        inner: R, image_size: Vector2<usize>, bit_depth: u8, color_type: ColorType,
    ) -> BaseDefilterer<R> {
        let (bytes_per_row, sub_pitch) = match bit_depth {
            1 | 2 | 4 => (DivCeil::div_ceil(image_size.x, 8 / bit_depth as usize), 1),
            8 => (
                image_size.x * color_type.channel_count(),
                color_type.channel_count(),
            ),
            16 => (
                image_size.x * color_type.channel_count() * 2,
                color_type.channel_count() * 2,
            ),
            _ => unreachable!(),
        };

        BaseDefilterer {
            bytes_per_row,
            height: image_size.y,
            inner,
            prev_row_data: Vec::new(),
            row_byte_index: 0,
            row_data: Vec::new(),
            row_index: 0,
            row_prefix: None,
            sub_pitch,
        }
    }
}

impl<R: Read> Read for BaseDefilterer<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.bytes_per_row == 0 || self.row_index == self.height || buf.is_empty() {
            return Ok(0);
        }

        // Read the row prefix byte. This determines how bytes are filtered for the rest of the row.
        let row_prefix = match self.row_prefix {
            None => {
                let b = self.inner.read_u8()?;
                self.row_prefix = Some(b);
                b
            },
            Some(b) => b,
        };

        // Read raw data from the inner stream.
        let n_to_read = std::cmp::min(buf.len(), self.bytes_per_row - self.row_byte_index);
        let n_read = self.inner.read(&mut buf[..n_to_read])?;
        if n_read == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
        }
        assert!(n_read <= n_to_read);
        self.row_data.extend(&buf[..n_read]);

        // De-filter the bytes that we just read.
        for i in 0..n_read {
            match row_prefix {
                0 /* None */ => (),
                1 /* Sub */ => {
                    let left = match (self.row_byte_index + i).checked_sub(self.sub_pitch) {
                        None => 0,
                        Some(j) => self.row_data[j],
                    };
                    buf[i] = buf[i].wrapping_add(left);
                },
                2 /* Up */ => {
                    let above = match self.row_index {
                        0 => 0,
                        _ => self.prev_row_data[self.row_byte_index + i],
                    };
                    buf[i] = buf[i].wrapping_add(above);
                },
                3 /* Average */ => {
                    let left = match (self.row_byte_index + i).checked_sub(self.sub_pitch) {
                        None => 0,
                        Some(j) => self.row_data[j],
                    };
                    let above = match self.row_index {
                        0 => 0,
                        _ => self.prev_row_data[self.row_byte_index + i],
                    };
                    let average = ((left as u16 + above as u16) / 2) as u8;
                    buf[i] = buf[i].wrapping_add(average);
                },
                4 /* Paeth */ => {
                    let (left, above, corner) = match ((self.row_byte_index + i).checked_sub(self.sub_pitch), self.row_index) {
                        (None, 0) => (0, 0, 0),
                        (None, _) => (0, self.prev_row_data[self.row_byte_index + i], 0),
                        (Some(j), 0) => (self.row_data[j], 0, 0),
                        (Some(j), _) => (self.row_data[j], self.prev_row_data[self.row_byte_index + i], self.prev_row_data[j]),
                    };
                    buf[i] = buf[i].wrapping_add(paeth(left, above, corner));
                },
                _ => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        Error::FilterByte { raw: row_prefix },
                    ))
                },
            }
        }

        // Advance to the next row.
        self.row_byte_index += n_read;
        if self.row_byte_index == self.bytes_per_row {
            self.row_byte_index = 0;
            self.row_index += 1;
            self.row_prefix = None;
            std::mem::swap(&mut self.row_data, &mut self.prev_row_data);
            self.row_data.clear();
        }

        Ok(n_read)
    }
}

/// Reverses pixel filtering when decoding a PNG stream. Includes all supported filter methods.
pub enum Defilterer<R: Read> {
    Base(BaseDefilterer<R>),
}

impl<R: Read> Defilterer<R> {
    pub fn into_inner(self) -> R {
        match self {
            Defilterer::Base(f) => f.into_inner(),
        }
    }

    pub fn new(
        filter_method: FilterMethod, inner: R, image_size: Vector2<usize>, bit_depth: u8,
        color_type: ColorType,
    ) -> Defilterer<R> {
        match filter_method {
            FilterMethod::Base => Defilterer::Base(BaseDefilterer::new(
                inner, image_size, bit_depth, color_type,
            )),
        }
    }
}

impl<R: Read> Read for Defilterer<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match *self {
            Defilterer::Base(ref mut f) => f.read(buf),
        }
    }
}

/// `PaethPredictor` algorithm as described at
/// <http://www.libpng.org/pub/png/spec/1.2/PNG-Filters.html>.
fn paeth(a: u8, b: u8, c: u8) -> u8 {
    let p = a as i16 + b as i16 - c as i16;
    let pa = (p - a as i16).abs();
    let pb = (p - b as i16).abs();
    let pc = (p - c as i16).abs();
    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}
