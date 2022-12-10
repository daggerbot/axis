/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::io::Read;

const STACK_BUFFER_LEN: usize = 4096;

/// Reader extensions.
pub trait ReadExt: Read {
    /// Reads and discards up to `n` bytes.
    fn skip(&mut self, n: usize) -> std::io::Result<usize> {
        if n == 0 {
            return Ok(0);
        }
        let mut buf = [0; STACK_BUFFER_LEN];
        let n_to_read = std::cmp::min(n, buf.len());
        let n_read = self.read(&mut buf[..n_to_read])?;
        assert!(n_read <= n_to_read);
        Ok(n_read)
    }

    /// Skips exactly `n` bytes of input. Returns an error if the end of the stream is reached
    /// before `n` bytes are read.
    fn skip_exact(&mut self, n: usize) -> std::io::Result<()> {
        let mut total = 0;
        while total < n {
            let n_read = self.skip(n - total)?;
            if n_read == 0 {
                return Err(std::io::Error::from(std::io::ErrorKind::UnexpectedEof));
            }
            total += n_read;
        }
        Ok(())
    }

    /// Discards all input until the end of the stream is reached.
    fn skip_to_end(&mut self) -> std::io::Result<()> {
        loop {
            if self.skip(STACK_BUFFER_LEN)? == 0 {
                return Ok(());
            }
        }
    }
}

impl<R: Read> ReadExt for R {}
