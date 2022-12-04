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

use byteorder::{WriteBytesExt, BE};
use crc32fast::Hasher;

/// PNG chunk ID.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ChunkId {
    raw: [u8; 4],
}

impl ChunkId {
    /// ID for the required chunk(s) containing the image's pixel data.
    pub const IDAT: ChunkId = unsafe { ChunkId::from_unchecked("IDAT") };
    /// ID for the required chunk at the end of any PNG stream.
    pub const IEND: ChunkId = unsafe { ChunkId::from_unchecked("IEND") };
    /// ID for the required chunk immediately following the file signature.
    pub const IHDR: ChunkId = unsafe { ChunkId::from_unchecked("IHDR") };
    /// ID for the chunk containing the palette, which is required if the image's color type is
    /// indexed.
    pub const PLTE: ChunkId = unsafe { ChunkId::from_unchecked("PLTE") };
    /// ID for the chunk containing transparency data if the color space does not include an alpha
    /// channel.
    #[allow(non_upper_case_globals)]
    pub const tRNS: ChunkId = unsafe { ChunkId::from_unchecked("tRNS") };

    /// Returns the chunk ID as a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.raw
    }

    /// Returns the chunk ID as an array of bytes.
    pub const fn as_raw(self) -> [u8; 4] {
        self.raw
    }

    /// Returns the chunk ID as a string.
    pub fn as_str(&self) -> &str {
        // All valid chunk IDs are valid UTF-8. Invalid chunk IDs can only be acquired privately or
        // by unsafe means, so this function can be considered safe.
        unsafe { std::str::from_utf8_unchecked(self.as_bytes()) }
    }

    /// Returns true if the chunk is ancillary, as indicated by the first letter being lower-case.
    /// An ancillary chunk is not required to properly display the image.
    pub fn is_ancillary(self) -> bool {
        char::from(self.raw[0]).is_lowercase()
    }

    /// Returns true if the chunk is critical, as indicated by the first letter being upper-case. A
    /// critical chunk is required to properly display the image.
    pub fn is_critical(self) -> bool {
        char::from(self.raw[0]).is_uppercase()
    }

    /// Returns true if the chunk is privately specified, as indicated by the second letter being
    /// lower-case.
    pub fn is_private(self) -> bool {
        char::from(self.raw[1]).is_lowercase()
    }

    /// Returns true if the chunk is publicly specified, as indicated by the second letter being
    /// upper-case.
    pub fn is_public(self) -> bool {
        char::from(self.raw[1]).is_uppercase()
    }

    /// Returns true if the chunk is safe to copy as-is when it is not understood by a decoder or an
    /// editor. This is indicated by the fourth letter being lower-case.
    pub fn is_safe_to_copy(self) -> bool {
        char::from(self.raw[3]).is_lowercase()
    }
}

impl ChunkId {
    const unsafe fn from_unchecked(s: &str) -> ChunkId {
        let bytes = s.as_bytes();
        ChunkId {
            raw: [bytes[0], bytes[1], bytes[2], bytes[3]],
        }
    }
}

impl AsRef<[u8]> for ChunkId {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl AsRef<str> for ChunkId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Display for ChunkId {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fmt.write_str(self.as_str())
    }
}

impl Into<[u8; 4]> for ChunkId {
    fn into(self) -> [u8; 4] {
        self.as_raw()
    }
}

impl Into<Vec<u8>> for ChunkId {
    fn into(self) -> Vec<u8> {
        Vec::from(self.as_bytes())
    }
}

impl Into<String> for ChunkId {
    fn into(self) -> String {
        String::from(self.as_str())
    }
}

impl<'a> TryFrom<[u8; 4]> for ChunkId {
    type Error = InvalidChunkId;

    fn try_from(array: [u8; 4]) -> Result<ChunkId, InvalidChunkId> {
        ChunkId::try_from(&array[..])
    }
}

impl<'a> TryFrom<&'a [u8]> for ChunkId {
    type Error = InvalidChunkId;

    fn try_from(bytes: &'a [u8]) -> Result<ChunkId, InvalidChunkId> {
        if bytes.len() != 4 {
            return Err(InvalidChunkId::Length(bytes.len()));
        }
        let mut raw = [0; 4];
        for i in 0..4 {
            raw[i] = bytes[i];
        }
        for i in 0..4 {
            match raw[i] {
                b'A'..=b'Z' | b'a'..=b'z' => (),
                _ => return Err(InvalidChunkId::Bytes(raw)),
            }
        }
        Ok(ChunkId { raw })
    }
}

impl<'a> TryFrom<&'a str> for ChunkId {
    type Error = InvalidChunkId;

    fn try_from(s: &'a str) -> Result<ChunkId, InvalidChunkId> {
        ChunkId::try_from(s.as_bytes())
    }
}

/// Raised when an invalid PNG chunk ID is encountered.
#[derive(Clone, Copy, Debug)]
pub enum InvalidChunkId {
    Bytes([u8; 4]),
    Length(usize),
}

impl InvalidChunkId {
    const DESCRIPTION: &'static str = "invalid png chunk id";
}

impl Display for InvalidChunkId {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        match *self {
            InvalidChunkId::Bytes(bytes) => write!(
                fmt,
                "{}: {:02x} {:02x} {:02x} {:02x}",
                Self::DESCRIPTION,
                bytes[0],
                bytes[1],
                bytes[2],
                bytes[3]
            ),
            InvalidChunkId::Length(len) => {
                write!(fmt, "{}: invalid length: {}", Self::DESCRIPTION, len)
            },
        }
    }
}

impl Error for InvalidChunkId {
    fn description(&self) -> &str {
        Self::DESCRIPTION
    }
}

/// Writes PNG chunks.
pub struct ChunkWriter<W: Write> {
    chunk_id: ChunkId,
    crc: Hasher,
    data: Vec<u8>,
    inner: W,
    max_len: Option<usize>,
    progressive: bool,
}

impl<W: Write> ChunkWriter<W> {
    /// Finishes writing the chunk and returns the inner writer.
    pub fn finish(mut self) -> std::io::Result<W> {
        self.write_chunk()?;
        Ok(self.inner)
    }

    /// Constructs a new PNG chunk writer.
    pub fn new(inner: W, chunk_id: ChunkId) -> ChunkWriter<W> {
        ChunkWriter {
            chunk_id,
            crc: init_crc(chunk_id),
            data: Vec::new(),
            inner,
            max_len: None,
            progressive: false,
        }
    }

    /// Constructs a PNG chunk writer which breaks up a stream into multiple chunks if they exceed a
    /// specified size.
    pub fn new_progressive(inner: W, chunk_id: ChunkId, max_len: usize) -> ChunkWriter<W> {
        ChunkWriter {
            chunk_id,
            crc: init_crc(chunk_id),
            data: Vec::new(),
            inner,
            max_len: Some(max_len),
            progressive: true,
        }
    }
}

impl<W: Write> ChunkWriter<W> {
    fn write_chunk(&mut self) -> std::io::Result<()> {
        let len = match u32::try_from(self.data.len()) {
            Ok(n) => n,
            Err(err) => return Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
        };
        let crc = std::mem::replace(&mut self.crc, init_crc(self.chunk_id)).finalize();

        self.inner.write_u32::<BE>(len)?;
        self.inner.write_all(self.chunk_id.as_bytes())?;
        self.inner.write_all(&self.data[..])?;
        self.inner.write_u32::<BE>(crc)?;

        self.data.clear();
        Ok(())
    }
}

impl<W: Write> Write for ChunkWriter<W> {
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }
        let len = match self.max_len {
            None => buf.len(),
            Some(max_len) => {
                let mut len = std::cmp::min(buf.len(), max_len - self.data.len());
                if len == 0 {
                    if self.progressive {
                        self.write_chunk()?;
                        len = std::cmp::min(buf.len(), max_len);
                    } else {
                        return Ok(0);
                    }
                }
                len
            },
        };

        let n = self.data.write(&buf[..len])?;
        self.crc.update(&buf[..n]);
        Ok(n)
    }
}

/// Initializes the CRC hasher.
fn init_crc(chunk_id: ChunkId) -> Hasher {
    let mut hasher = Hasher::new_with_initial(0);
    hasher.update(chunk_id.as_bytes());
    hasher
}
