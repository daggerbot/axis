/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::fmt::{Display, Formatter};
use std::io::{Read, Write};

use byteorder::{ByteOrder, ReadBytesExt, WriteBytesExt, BE};
use crc32fast::Hasher;
use peekread::PeekRead;

use crate::codec::png::Error;
use crate::io::ReadExt;

/// 4-letter PNG chunk ID. This is the second field found in the chunk header and determines what a
/// chunk's data stream is used for.
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
        unsafe {
            std::str::from_utf8_unchecked(self.as_bytes())
        }
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
    type Error = Error;

    fn try_from(array: [u8; 4]) -> Result<ChunkId, Error> {
        ChunkId::try_from(&array[..])
    }
}

impl<'a> TryFrom<&'a [u8]> for ChunkId {
    type Error = Error;

    fn try_from(bytes: &'a [u8]) -> Result<ChunkId, Error> {
        if bytes.len() != 4 {
            return Err(Error::ChunkIdLen { len: bytes.len() });
        }
        let mut raw = [0; 4];
        for i in 0..4 {
            raw[i] = bytes[i];
        }
        for i in 0..4 {
            match raw[i] {
                b'A'..=b'Z' | b'a'..=b'z' => (),
                _ => return Err(Error::ChunkId { bytes: raw }),
            }
        }
        Ok(ChunkId { raw })
    }
}

impl<'a> TryFrom<&'a str> for ChunkId {
    type Error = Error;

    fn try_from(s: &'a str) -> Result<ChunkId, Error> {
        ChunkId::try_from(s.as_bytes())
    }
}

/// Reads the data stream in a PNG chunk and checks the CRC at the end.
///
/// The caller must call [`finish`] when reading is done. If the chunk reader is simply dropped, the
/// CRC is ignored.
pub struct ChunkReader<R: Read> {
    chunk_id: ChunkId,
    crc: Hasher,
    inner: R,
    len: u32,
    pos: u32,
}

impl<R: Read> ChunkReader<R> {
    /// Returns the chunk ID.
    pub fn chunk_id(&self) -> ChunkId {
        self.chunk_id
    }

    /// Returns the length of the entire chunk, including bytes that have already been read.
    pub fn chunk_len(&self) -> u32 {
        self.len
    }

    /// Returns true if the end of the chunk has been reached.
    pub fn eof(&self) -> bool {
        self.pos == self.len
    }

    /// Reads to the end of the chunk, checks the CRC, and returns the inner reader.
    pub fn finish(mut self) -> Result<R, Error> {
        self.check_crc()?;
        Ok(self.inner)
    }

    /// Begins reading a chunk.
    ///
    /// Fails if the end of the stream is reached unexpectedly or if an invalid chunk ID is
    /// encountered.
    pub fn new(mut inner: R) -> Result<ChunkReader<R>, Error> {
        let len = inner.read_u32::<BE>()?;
        let mut chunk_id_bytes = [0; 4];
        inner.read_exact(&mut chunk_id_bytes[..])?;
        let chunk_id = ChunkId::try_from(chunk_id_bytes)?;

        Ok(ChunkReader {
            chunk_id,
            crc: init_crc(chunk_id),
            inner,
            len,
            pos: 0,
        })
    }

    /// Returns the number of bytes remaining in the chunk's data stream.
    pub fn remaining(&self) -> u32 {
        self.len - self.pos
    }
}

impl<R: Read> ChunkReader<R> {
    /// Reads to the end of the chunk and checks the CRC. Because the hasher's `finalize()` function
    /// consumes the hasher, this results in the hasher being reset. The implementation of
    /// `ProgressiveChunkReader` takes advantage of this.
    fn check_crc(&mut self) -> Result<(), Error> {
        self.skip_to_end()?;
        let crc = self.inner.read_u32::<BE>()?;
        if self.crc.clone().finalize() != crc {
            return Err(Error::Crc);
        }
        Ok(())
    }
}

impl<R: Read> Read for ChunkReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let n_to_read = std::cmp::min(buf.len(),
                                      usize::try_from(self.remaining()).unwrap_or(usize::MAX));
        if n_to_read == 0 {
            return Ok(0);
        }
        let n_read = self.inner.read(&mut buf[..n_to_read])?;
        if n_read == 0 {
            return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
        }
        assert!(n_read <= n_to_read);
        self.crc.update(&buf[..n_read]);
        self.pos += n_read as u32;
        Ok(n_read)
    }
}

/// Reads consecutive chunks with the same ID as a single data stream.
///
/// This is provided because some data streams, particularly `IDAT`, may be split into multiple
/// chunks so the encoder doesn't have to know the length of an entire data stream before writing.
/// The `IDAT` stream is compressed, and the encoder would have no way to know how long the
/// compressed stream would be prior to writing unless it compressed the entire stream twice.
///
/// The inner reader must implement [`PeekRead`] rather than simply [`Read`]. This is because the
/// progressive chunk reader must look ahead 8 bytes to determine whether the next chunk has the
/// same ID as the current one.
pub struct ProgressiveChunkReader<R: PeekRead> {
    chunk: ChunkReader<R>,
}

impl<R: PeekRead> ProgressiveChunkReader<R> {
    /// Returns the chunk ID.
    pub fn chunk_id(&self) -> ChunkId {
        self.chunk.chunk_id
    }

    /// Constructs a progressive chunk reader which will interpret subsequent chunks with the same
    /// ID as being part of the same data stream.
    pub fn new(chunk: ChunkReader<R>) -> ProgressiveChunkReader<R> {
        ProgressiveChunkReader { chunk }
    }

    /// Finishes reading all subsequent chunks with the same ID and returns the inner reader.
    pub fn finish(mut self) -> Result<R, Error> {
        self.skip_to_end()?;
        Ok(self.chunk.inner)
    }
}

impl<R: PeekRead> Read for ProgressiveChunkReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        loop {
            let n_read = self.chunk.read(buf)?;
            if n_read != 0 {
                return Ok(n_read);
            }

            // We've reached the end of the chunk. Check if the next chunk has the same ID.
            match self.chunk.check_crc() {
                Ok(()) => (),
                Err(Error::Crc) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        Error::Crc,
                    ))
                },
                Err(err) => return Err(std::io::Error::new(std::io::ErrorKind::Other, err)),
            };
            let mut chunk_header = [0; 8];
            self.chunk.inner.peek().read_exact(&mut chunk_header[..])?;
            let chunk_id = match ChunkId::try_from(&chunk_header[4..8]) {
                Ok(chunk_id) => chunk_id,
                Err(_) => return Ok(0),
            };
            if chunk_id != self.chunk.chunk_id {
                return Ok(0);
            }

            // We've confirmed that the next chunk has the same ID. Let's start reading from that
            // chunk now.
            self.chunk.crc = init_crc(chunk_id);
            self.chunk.len = <BE as ByteOrder>::read_u32(&chunk_header[..4]);
            self.chunk.pos = 0;
        }
    }
}

/// Writes a PNG chunk.
///
/// The user must call [`finish`] to complete writing the chunk, or else the entire chunk, including
/// the header, will be discarded. This is because the length of the chunk must be known before
/// anything can be written to the inner writer.
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

    /// Constructs a PNG chunk writer which breaks up a data stream into multiple chunks if they
    /// exceed a specified size. The is useful particularly for writing the `IDAT` stream, which is
    /// allowed to be broken up into multiple consecutive chunks.
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
    /// Writes the chunk to the inner writer and resets the state of the chunk writer.
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

        let n_to_write = match self.max_len {
            None => buf.len(),
            Some(max_len) => {
                let mut n_to_write = std::cmp::min(buf.len(), max_len - self.data.len());
                if n_to_write == 0 {
                    if self.progressive {
                        self.write_chunk()?;
                        n_to_write = std::cmp::min(buf.len(), max_len);
                    } else {
                        return Ok(0);
                    }
                }
                n_to_write
            },
        };

        let n_written = self.data.write(&buf[..n_to_write])?;
        self.crc.update(&buf[..n_written]);
        Ok(n_written)
    }
}

/// Initializes the CRC hasher.
fn init_crc(chunk_id: ChunkId) -> Hasher {
    let mut hasher = Hasher::new_with_initial(0);
    hasher.update(chunk_id.as_bytes());
    hasher
}
