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

use flate2::write::ZlibEncoder;

/// Enumeration of PNG compression methods.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(u8)]
pub enum CompressionMethod {
    Zlib = 0,
}

impl CompressionMethod {
    const fn description(self) -> &'static str {
        match self {
            CompressionMethod::Zlib => "zlib",
        }
    }
}

impl Display for CompressionMethod {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fmt.write_str(self.description())
    }
}

impl TryFrom<u8> for CompressionMethod {
    type Error = InvalidCompressionMethod;

    fn try_from(byte: u8) -> Result<CompressionMethod, InvalidCompressionMethod> {
        match byte {
            0 => Ok(CompressionMethod::Zlib),
            _ => Err(InvalidCompressionMethod(byte)),
        }
    }
}

/// Raised when an invalid PNG compression method is encountered.
#[derive(Clone, Copy, Debug)]
pub struct InvalidCompressionMethod(pub u8);

impl InvalidCompressionMethod {
    const DESCRIPTION: &'static str = "invalid png compression method";
}

impl Display for InvalidCompressionMethod {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        write!(fmt, "{}: {}", Self::DESCRIPTION, self.0)
    }
}

impl Error for InvalidCompressionMethod {
    fn description(&self) -> &str { Self::DESCRIPTION }
}

/// Stream wrapper for compressing data.
pub enum Compressor<W: Write> {
    Zlib(ZlibEncoder<W>),
}

impl<W: Write> Compressor<W> {
    /// Finishes compressing and returns the inner writer.
    pub fn finish(self) -> std::io::Result<W> {
        match self {
            Compressor::Zlib(zlib) => zlib.finish(),
        }
    }

    /// Constructs a compressor with the specified compression method.
    pub fn new(inner: W, compression_method: CompressionMethod) -> Compressor<W> {
        match compression_method {
            CompressionMethod::Zlib => {
                let inner = ZlibEncoder::new(inner, flate2::Compression::best());
                Compressor::Zlib(inner)
            },
        }
    }
}

impl<W: Write> Write for Compressor<W> {
    fn flush(&mut self) -> std::io::Result<()> {
        match *self {
            Compressor::Zlib(ref mut inner) => inner.flush(),
        }
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match *self {
            Compressor::Zlib(ref mut inner) => inner.write(buf),
        }
    }
}
