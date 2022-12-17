/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::fmt::{Display, Formatter};
use std::io::{Read, Write};

use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;

use crate::codec::png::Error;

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
    type Error = Error;

    fn try_from(raw: u8) -> Result<CompressionMethod, Error> {
        match raw {
            0 => Ok(CompressionMethod::Zlib),
            _ => Err(Error::CompressionMethod { raw }),
        }
    }
}

/// Wrapper which compressed data using the corresponding PNG compression method.
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
                Compressor::Zlib(ZlibEncoder::new(inner, flate2::Compression::best()))
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

/// Wrapper which decompresses data using the corresponding PNG compression method.
pub enum Decompressor<R: Read> {
    Zlib(ZlibDecoder<R>),
}

impl<R: Read> Decompressor<R> {
    /// Returns the underlying reader. Any unread data in the compressed stream is discarded.
    pub fn into_inner(self) -> R {
        match self {
            Decompressor::Zlib(r) => r.into_inner(),
        }
    }

    /// Constructs a decompressor with the specified compression method.
    pub fn new(inner: R, compression_method: CompressionMethod) -> Decompressor<R> {
        match compression_method {
            CompressionMethod::Zlib => Decompressor::Zlib(ZlibDecoder::new(inner)),
        }
    }
}

impl<R: Read> Read for Decompressor<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match *self {
            Decompressor::Zlib(ref mut r) => r.read(buf),
        }
    }
}
