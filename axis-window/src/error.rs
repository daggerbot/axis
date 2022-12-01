/*
 * Copyright (c) 2022 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::borrow::Cow;
use std::fmt::{Display, Formatter};

/// Result type used throughout the crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Enumeration of error kinds used throughout the crate.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    ArithmeticError,
    ConnectionFailed,
    EncodingError,
    IncompatibleResource,
    InvalidArgument,
    IoError,
    RequestFailed,
    ResourceExpired,
    SyncError,
    SystemError,
    UnsupportedPlatform,
}

impl ErrorKind {
    /// Returns a brief description of the error.
    pub const fn description(self) -> &'static str {
        match self {
            ErrorKind::ArithmeticError => "arithmetic error",
            ErrorKind::ConnectionFailed => "connection failed",
            ErrorKind::EncodingError => "encoding error",
            ErrorKind::IncompatibleResource => "incompatible resource",
            ErrorKind::InvalidArgument => "invalid argument",
            ErrorKind::IoError => "i/o error",
            ErrorKind::RequestFailed => "request failed",
            ErrorKind::ResourceExpired => "resource expired",
            ErrorKind::SyncError => "synchronization error",
            ErrorKind::SystemError => "system error",
            ErrorKind::UnsupportedPlatform => "unsupported platform",
        }
    }
}

/// Error type used throughout the crate.
#[derive(Debug)]
pub struct Error {
    detail: Option<Cow<'static, str>>,
    kind: ErrorKind,
    source: Option<Box<dyn 'static + Send + Sync + std::error::Error>>,
}

impl Error {
    /// Returns a string describing any additional details about the error.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_ref().map(|s| s.as_ref())
    }

    /// Returns the error kind.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Sets the detail string to a static string.
    pub fn with_detail(self, detail: &'static str) -> Error {
        Error {
            detail: Some(Cow::Borrowed(detail)),
            ..self
        }
    }

    /// Sets the detail string to an owned string.
    pub fn with_detail_string(self, detail: String) -> Error {
        Error {
            detail: Some(Cow::Owned(detail)),
            ..self
        }
    }

    /// Sets the source error.
    pub fn with_source<E: 'static + Send + Sync + std::error::Error>(self, source: E) -> Error {
        Error {
            source: Some(Box::new(source)),
            ..self
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            detail: None,
            kind,
            source: None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, fmt: &mut Formatter) -> std::fmt::Result {
        fmt.write_str(self.kind.description())?;
        if let Some(ref detail) = self.detail {
            write!(fmt, " ({})", detail)?;
        }
        if let Some(ref source) = self.source {
            write!(fmt, ": {}", source)?;
        }
        Ok(())
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(source: std::ffi::NulError) -> Error {
        err!(EncodingError: source)
    }
}

impl From<std::num::TryFromIntError> for Error {
    fn from(source: std::num::TryFromIntError) -> Error {
        err!(ArithmeticError: source)
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(source: std::sync::PoisonError<T>) -> Error {
        err!(SyncError{"{}", source})
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        self.kind.description()
    }

    fn source(&self) -> Option<&(dyn 'static + std::error::Error)> {
        match self.source {
            None => None,
            Some(ref source) => Some(source.as_ref()),
        }
    }
}
