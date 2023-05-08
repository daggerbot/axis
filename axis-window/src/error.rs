/*
 * Copyright (c) 2023 Martin Mills <daggerbot@gmail.com>
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

use std::borrow::Cow;
use std::fmt::{Display, Formatter};

/// Generic `axis-window` result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Enumeration of `axis-window` error kinds.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ErrorKind {
    ConnectionFailed,
    EncodingError,
    IncompatibleResource,
    InvalidArgument,
    IoError,
    LibraryError,
    LockError,
    RequestFailed,
    ResourceExpired,
    RuntimeError,
}

impl ErrorKind {
    /// Returns a brief message describing the error.
    pub fn brief(self) -> &'static str {
        match self {
            ErrorKind::ConnectionFailed => "connection failed",
            ErrorKind::EncodingError => "encoding error",
            ErrorKind::IncompatibleResource => "incompatible resource",
            ErrorKind::InvalidArgument => "invalid argument",
            ErrorKind::IoError => "I/O error",
            ErrorKind::LibraryError => "library error",
            ErrorKind::LockError => "lock error",
            ErrorKind::RequestFailed => "request failed",
            ErrorKind::ResourceExpired => "resource expired",
            ErrorKind::RuntimeError => "runtime error",
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(self.brief())
    }
}

/// Generic error type.
#[derive(Debug)]
pub struct Error {
    detail: Option<Cow<'static, str>>,
    kind: ErrorKind,
    source: Option<Box<dyn 'static + std::error::Error>>,
}

impl Error {
    /// Returns a brief message describing the error.
    pub fn brief(&self) -> &'static str {
        self.kind.brief()
    }

    /// Returns a string describing more details about the error.
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_ref().map(|s| s.as_ref())
    }

    /// Returns the error kind.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// Sets the error's source if the parameter is `Some`.
    pub fn maybe_with_source<E: 'static + std::error::Error>(self, source: Option<E>) -> Error {
        Error {
            source: match source {
                None => self.source,
                Some(err) => Some(Box::new(err)),
            },
            ..self
        }
    }

    /// Sets the error's detail message to a static string.
    pub fn with_detail(self, detail: &'static str) -> Error {
        Error {
            detail: Some(Cow::Borrowed(detail)),
            ..self
        }
    }

    /// Sets the error's detail message to an owned string.
    pub fn with_detail_string(self, detail: String) -> Error {
        Error {
            detail: Some(Cow::Owned(detail)),
            ..self
        }
    }

    /// Sets the error's source.
    pub fn with_source<E: 'static + std::error::Error>(self, source: E) -> Error {
        Error {
            source: Some(Box::new(source)),
            ..self
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.write_str(self.brief())?;
        if let Some(ref detail) = self.detail {
            write!(f, " ({})", detail)?;
        }
        if let Some(ref source) = self.source {
            write!(f, ": {}", source)?;
        }
        Ok(())
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

impl From<std::ffi::NulError> for Error {
    fn from(source: std::ffi::NulError) -> Error {
        err!(EncodingError: source)
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(source: std::sync::PoisonError<T>) -> Error {
        err!(LockError{"{}", source})
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        self.brief()
    }

    fn source(&self) -> Option<&(dyn 'static + std::error::Error)> {
        self.source.as_ref().map(|e| e.as_ref())
    }
}
