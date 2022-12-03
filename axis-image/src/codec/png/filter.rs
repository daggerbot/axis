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

use byteorder::WriteBytesExt;
use math::{DivCeil, Vector2};

use crate::codec::png::ColorType;

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
    type Error = InvalidFilterMethod;

    fn try_from(byte: u8) -> Result<FilterMethod, InvalidFilterMethod> {
        match byte {
            0 => Ok(FilterMethod::Base),
            _ => Err(InvalidFilterMethod(byte)),
        }
    }
}

/// Raised when an invalid PNG filter method is encountered.
#[derive(Clone, Copy, Debug)]
pub struct InvalidFilterMethod(pub u8);

impl InvalidFilterMethod {
    const DESCRIPTION: &'static str = "invalid png filter method";
}

impl Display for InvalidFilterMethod {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}: {}", Self::DESCRIPTION, self.0)
    }
}

impl Error for InvalidFilterMethod {
    fn description(&self) -> &str {
        Self::DESCRIPTION
    }
}

/// Filters bytes for better compression.
pub struct BaseFilterer<W: Write> {
    inner: W,
    pos: Vector2<usize>,
    size: Vector2<usize>,
}

impl<W: Write> BaseFilterer<W> {
    pub fn finish(self) -> W {
        self.inner
    }

    pub fn new(
        inner: W, image_size: Vector2<usize>, bit_depth: u8, color_type: ColorType,
    ) -> BaseFilterer<W> {
        let row_len = match bit_depth {
            1 | 2 | 4 => DivCeil::div_ceil(image_size.x, 8 / bit_depth as usize),
            8 => image_size.x * color_type.channel_count(),
            16 => image_size.x * color_type.channel_count() * 2,
            _ => unreachable!(),
        };

        BaseFilterer {
            inner,
            pos: Vector2::new(0, 0),
            size: Vector2::new(row_len, image_size.y),
        }
    }
}

impl<W: Write> Write for BaseFilterer<W> {
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        // TODO: Do actual filtering instead of just using filter type 0 for every row.
        if self.size.x == 0 || self.pos.y == self.size.y || buf.is_empty() {
            return Ok(0);
        }
        if self.pos.x == 0 {
            self.inner.write_u8(0)?;
        }
        let len = std::cmp::min(buf.len(), self.size.x - self.pos.x);
        let result = self.inner.write(&buf[..len])?;
        self.pos.x += result;
        if self.pos.x == self.size.x {
            self.pos.x = 0;
            self.pos.y += 1;
        }
        Ok(result)
    }
}
