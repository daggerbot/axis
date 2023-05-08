/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::io::{Read, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};

use libc::{size_t, ssize_t};

/// Reads from a POSIX pipe.
pub struct PipeReader(OwnedFd);

impl AsFd for PipeReader {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for PipeReader {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl FromRawFd for PipeReader {
    unsafe fn from_raw_fd(fd: RawFd) -> PipeReader {
        PipeReader(OwnedFd::from_raw_fd(fd))
    }
}

impl Read for PipeReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        read(self.as_raw_fd(), buf)
    }
}

/// Writes to a POSIX pipe.
pub struct PipeWriter(OwnedFd);

impl AsFd for PipeWriter {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for PipeWriter {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

impl FromRawFd for PipeWriter {
    unsafe fn from_raw_fd(fd: RawFd) -> PipeWriter {
        PipeWriter(OwnedFd::from_raw_fd(fd))
    }
}

impl Write for PipeWriter {
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        write(self.as_raw_fd(), buf)
    }
}

/// Opens an anonymous POSIX pipe.
pub fn pipe() -> std::io::Result<(PipeReader, PipeWriter)> {
    unsafe {
        let mut fds = [-1, -1];
        let result = libc::pipe(fds.as_mut_ptr());

        if result == 0 {
            Ok((PipeReader::from_raw_fd(fds[0]), PipeWriter::from_raw_fd(fds[1])))
        } else {
            Err(std::io::Error::last_os_error())
        }
    }
}

/// Reads from a file descriptor.
pub fn read(fd: RawFd, buf: &mut [u8]) -> std::io::Result<usize> {
    let len = ssize_t::try_from(buf.len()).unwrap_or(ssize_t::MAX);
    let result;

    unsafe {
        result = libc::read(fd, buf.as_mut_ptr() as *mut _, len as size_t);
    }

    if result >= 0 {
        Ok(result as usize)
    } else {
        Err(std::io::Error::last_os_error())
    }
}

/// Writes to a file descriptor.
pub fn write(fd: RawFd, buf: &[u8]) -> std::io::Result<usize> {
    let len = ssize_t::try_from(buf.len()).unwrap_or(ssize_t::MAX);
    let result;

    unsafe {
        result = libc::write(fd, buf.as_ptr() as *const _, len as size_t);
    }

    if result >= 0 {
        Ok(result as usize)
    } else {
        Err(std::io::Error::last_os_error())
    }
}
